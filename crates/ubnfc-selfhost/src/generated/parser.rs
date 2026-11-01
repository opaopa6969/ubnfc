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
("UBNF",Some("AdapterExpression"))=>parser.r22_c(State::default()),
("UBNF",Some("AnyExpression"))=>parser.r23_c(State::default()),
("UBNF",Some("EofExpression"))=>parser.r24_c(State::default()),
("UBNF",Some("EmptyExpression"))=>parser.r25_c(State::default()),
("UBNF",Some("RuleDecl"))=>parser.r26_c(State::default()),
("UBNF",Some("Annotation"))=>parser.r27_c(State::default()),
("UBNF",Some("RootAnnotation"))=>parser.r28_c(State::default()),
("UBNF",Some("MappingAnnotation"))=>parser.r29_c(State::default()),
("UBNF",Some("EvalAnnotation"))=>parser.r30_c(State::default()),
("UBNF",Some("WhitespaceAnnotation"))=>parser.r31_c(State::default()),
("UBNF",Some("LexicalContextAnnotation"))=>parser.r32_c(State::default()),
("UBNF",Some("InterleaveAnnotation"))=>parser.r33_c(State::default()),
("UBNF",Some("BackrefAnnotation"))=>parser.r34_c(State::default()),
("UBNF",Some("ScopeTreeAnnotation"))=>parser.r35_c(State::default()),
("UBNF",Some("DeclaresAnnotation"))=>parser.r36_c(State::default()),
("UBNF",Some("CatalogAnnotation"))=>parser.r37_c(State::default()),
("UBNF",Some("LeftAssocAnnotation"))=>parser.r38_c(State::default()),
("UBNF",Some("RightAssocAnnotation"))=>parser.r39_c(State::default()),
("UBNF",Some("LongestChoiceAnnotation"))=>parser.r40_c(State::default()),
("UBNF",Some("UniqueLongestChoiceAnnotation"))=>parser.r41_c(State::default()),
("UBNF",Some("NamePredicateAnnotation"))=>parser.r42_c(State::default()),
("UBNF",Some("PredicateAnnotation"))=>parser.r43_c(State::default()),
("UBNF",Some("PredictiveChoiceAnnotation"))=>parser.r44_c(State::default()),
("UBNF",Some("PrecedenceAnnotation"))=>parser.r45_c(State::default()),
("UBNF",Some("DocAnnotation"))=>parser.r46_c(State::default()),
("UBNF",Some("RecoveryAnnotation"))=>parser.r47_c(State::default()),
("UBNF",Some("SkipAnnotation"))=>parser.r48_c(State::default()),
("UBNF",Some("CommonFieldAnnotation"))=>parser.r49_c(State::default()),
("UBNF",Some("EnumAnnotation"))=>parser.r50_c(State::default()),
("UBNF",Some("SimpleAnnotation"))=>parser.r51_c(State::default()),
("UBNF",Some("ChoiceBody"))=>parser.r52_c(State::default()),
("UBNF",Some("SequenceBody"))=>parser.r53_c(State::default()),
("UBNF",Some("AnnotatedElement"))=>parser.r54_c(State::default()),
("UBNF",Some("PostfixQuantifier"))=>parser.r55_c(State::default()),
("UBNF",Some("SeparatedBy"))=>parser.r56_c(State::default()),
("UBNF",Some("BoundedQuantifier"))=>parser.r57_c(State::default()),
("UBNF",Some("Digits"))=>parser.r58_c(State::default()),
("UBNF",Some("AtomicElement"))=>parser.r59_c(State::default()),
("UBNF",Some("OwnedCaptureElement"))=>parser.r60_c(State::default()),
("UBNF",Some("ReplayElement"))=>parser.r61_c(State::default()),
("UBNF",Some("CaptureScopeElement"))=>parser.r62_c(State::default()),
("UBNF",Some("CommitElement"))=>parser.r63_c(State::default()),
("UBNF",Some("LabelElement"))=>parser.r64_c(State::default()),
("UBNF",Some("CatchElement"))=>parser.r65_c(State::default()),
("UBNF",Some("GroupElement"))=>parser.r66_c(State::default()),
("UBNF",Some("OptionalElement"))=>parser.r67_c(State::default()),
("UBNF",Some("RepeatElement"))=>parser.r68_c(State::default()),
("UBNF",Some("ErrorElement"))=>parser.r69_c(State::default()),
("UBNF",Some("TerminalElement"))=>parser.r70_c(State::default()),
("UBNF",Some("QuantifiedRef"))=>parser.r71_c(State::default()),
("UBNF",Some("TypeofElement"))=>parser.r72_c(State::default()),
("UBNF",Some("DottedIdentifier"))=>parser.r73_c(State::default()),
("UBNF",Some("RuleBody"))=>parser.r74_c(State::default()),
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
let mut out=self.e34_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:6,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(6,out.diag);out
}
fn r7_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e39_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:7,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
self.depth-=1;out.diag=self.diag_rule(7,out.diag);out
}
fn r8_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e43_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:8,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'token'"]);}
self.depth-=1;out.diag=self.diag_rule(8,out.diag);out
}
fn r9_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e54_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:9,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
self.depth-=1;out.diag=self.diag_rule(9,out.diag);out
}
fn r10_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e57_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:10,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(10,out.diag);out
}
fn r11_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e63_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:11,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalSequenceParser"]);}
self.depth-=1;out.diag=self.diag_rule(11,out.diag);out
}
fn r12_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e66_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:12,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalElementParser"]);}
self.depth-=1;out.diag=self.diag_rule(12,out.diag);out
}
fn r13_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e70_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:13,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalSuffixParser"]);}
self.depth-=1;out.diag=self.diag_rule(13,out.diag);out
}
fn r14_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e85_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:14,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalAtomParser"]);}
self.depth-=1;out.diag=self.diag_rule(14,out.diag);out
}
fn r15_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e135_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:15,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'", "'('", "')'", "'NEGATION'", "'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'", "'CHAR_RANGE'", "','", "'CI'", "'ADAPTER'", "'version'", "'='", "'ANY'", "'EOF'", "'EMPTY'", "'XID_IDENTIFIER'"]);}
self.depth-=1;out.diag=self.diag_rule(15,out.diag);out
}
fn r16_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e148_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:16,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(16,out.diag);out
}
fn r17_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e153_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:17,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'NEGATION'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(17,out.diag);out
}
fn r18_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e158_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:18,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(18,out.diag);out
}
fn r19_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e163_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:19,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'NEGATIVE_LOOKAHEAD'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(19,out.diag);out
}
fn r20_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e168_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:20,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CHAR_RANGE'", "'('", "','", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(20,out.diag);out
}
fn r21_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e175_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:21,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CI'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(21,out.diag);out
}
fn r22_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e180_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:22,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'ADAPTER'", "'('", "','", "'version'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(22,out.diag);out
}
fn r23_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e189_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:23,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'ANY'"]);}
self.depth-=1;out.diag=self.diag_rule(23,out.diag);out
}
fn r24_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e191_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:24,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'EOF'"]);}
self.depth-=1;out.diag=self.diag_rule(24,out.diag);out
}
fn r25_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e193_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:25,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'EMPTY'"]);}
self.depth-=1;out.diag=self.diag_rule(25,out.diag);out
}
fn r26_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e195_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:26,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'::='", "';'"]);}
self.depth-=1;out.diag=self.diag_rule(26,out.diag);out
}
fn r27_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e202_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:27,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@root'", "'@mapping'", "'('", "')'", "'@eval'", "'@whitespace'", "'@interleave'", "'profile'", "'='", "'@lexicalContext'", "'tokens'", "'['", "']'", "'@backref'", "'name'", "'@scopeTree'", "'mode'", "'@declares'", "'symbol'", "'@catalog'", "'context'", "'@leftAssoc'", "'@rightAssoc'", "'@longestChoice'", "'@uniqueLongestChoice'", "'@namePredicate'", "'snapshot'", "','", "'version'", "'kind'", "'@predicate'", "'@predictiveChoice'", "'@precedence'", "'level'", "'@doc'", "'@recovery'", "'@skip'", "'@commonField'", "'@enum'", "'@'"]);}
self.depth-=1;out.diag=self.diag_rule(27,out.diag);out
}
fn r28_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e227_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:28,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@root'"]);}
self.depth-=1;out.diag=self.diag_rule(28,out.diag);out
}
fn r29_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e229_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:29,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@mapping'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(29,out.diag);out
}
fn r30_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e246_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:30,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@eval'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(30,out.diag);out
}
fn r31_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e255_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:31,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@whitespace'"]);}
self.depth-=1;out.diag=self.diag_rule(31,out.diag);out
}
fn r32_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e262_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:32,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@lexicalContext'", "'('", "'tokens'", "'='", "'['", "']'", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(32,out.diag);out
}
fn r33_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e291_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:33,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@interleave'", "'('", "'profile'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(33,out.diag);out
}
fn r34_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e298_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:34,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@backref'", "'('", "'name'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(34,out.diag);out
}
fn r35_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e305_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:35,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@scopeTree'", "'('", "'mode'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(35,out.diag);out
}
fn r36_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e312_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:36,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@declares'", "'('", "'symbol'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(36,out.diag);out
}
fn r37_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e325_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:37,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@catalog'", "'('", "'context'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(37,out.diag);out
}
fn r38_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e332_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:38,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@leftAssoc'"]);}
self.depth-=1;out.diag=self.diag_rule(38,out.diag);out
}
fn r39_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e334_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:39,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@rightAssoc'"]);}
self.depth-=1;out.diag=self.diag_rule(39,out.diag);out
}
fn r40_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e336_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:40,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@longestChoice'"]);}
self.depth-=1;out.diag=self.diag_rule(40,out.diag);out
}
fn r41_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e338_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:41,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@uniqueLongestChoice'"]);}
self.depth-=1;out.diag=self.diag_rule(41,out.diag);out
}
fn r42_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e340_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:42,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@namePredicate'", "'('", "'snapshot'", "'='", "','", "'version'", "'name'", "'kind'", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(42,out.diag);out
}
fn r43_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e359_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:43,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@predicate'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(43,out.diag);out
}
fn r44_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e364_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:44,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@predictiveChoice'"]);}
self.depth-=1;out.diag=self.diag_rule(44,out.diag);out
}
fn r45_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e366_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:45,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@precedence'", "'('", "'level'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(45,out.diag);out
}
fn r46_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e373_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:46,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@doc'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(46,out.diag);out
}
fn r47_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e378_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:47,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@recovery'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(47,out.diag);out
}
fn r48_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e389_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:48,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@skip'"]);}
self.depth-=1;out.diag=self.diag_rule(48,out.diag);out
}
fn r49_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e391_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:49,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@commonField'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(49,out.diag);out
}
fn r50_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e400_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:50,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@enum'"]);}
self.depth-=1;out.diag=self.diag_rule(50,out.diag);out
}
fn r51_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e402_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:51,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
self.depth-=1;out.diag=self.diag_rule(51,out.diag);out
}
fn r52_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e405_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:52,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["ChoiceBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(52,out.diag);out
}
fn r53_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e411_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:53,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["SequenceBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(53,out.diag);out
}
fn r54_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e414_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:54,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["AnnotatedElementParser"]);}
self.depth-=1;out.diag=self.diag_rule(54,out.diag);out
}
fn r55_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e428_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:55,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'+'", "'?'", "'*'"]);}
self.depth-=1;out.diag=self.diag_rule(55,out.diag);out
}
fn r56_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e432_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:56,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
self.depth-=1;out.diag=self.diag_rule(56,out.diag);out
}
fn r57_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e435_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:57,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(57,out.diag);out
}
fn r58_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e444_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:58,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["DigitsParser"]);}
self.depth-=1;out.diag=self.diag_rule(58,out.diag);out
}
fn r59_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e446_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:59,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CAPTURE('", "','", "')'", "'SAME_AS('", "'CAPTURE_SCOPE('", "'COMMIT('", "'LABEL('", "'CATCH('", "'('", "'['", "']'", "'{'", "'}'", "'ERROR'"]);}
self.depth-=1;out.diag=self.diag_rule(59,out.diag);out
}
fn r60_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e459_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:60,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CAPTURE('", "','", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(60,out.diag);out
}
fn r61_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e465_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:61,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'SAME_AS('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(61,out.diag);out
}
fn r62_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e469_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:62,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CAPTURE_SCOPE('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(62,out.diag);out
}
fn r63_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e473_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:63,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'COMMIT('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(63,out.diag);out
}
fn r64_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e481_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:64,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'LABEL('", "','", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(64,out.diag);out
}
fn r65_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e487_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:65,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CATCH('", "','", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(65,out.diag);out
}
fn r66_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e497_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:66,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(66,out.diag);out
}
fn r67_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e501_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:67,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'['", "']'"]);}
self.depth-=1;out.diag=self.diag_rule(67,out.diag);out
}
fn r68_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e505_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:68,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(68,out.diag);out
}
fn r69_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e509_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:69,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'ERROR'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(69,out.diag);out
}
fn r70_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e514_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:70,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["TerminalElementParser"]);}
self.depth-=1;out.diag=self.diag_rule(70,out.diag);out
}
fn r71_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e516_c(state);
if out.ok {
let mut scope_sites=self.take_scope();self.scope_captures(out.events,&[88],&mut scope_sites);for &(site,span) in &scope_sites { let name=self.text(span);if name.is_empty() {continue;}let offset_cp=self.cp(span[0]);let len_cp=self.cp(span[1])-offset_cp;
if site==88 {let mut targets=self.take_scope();self.scope_captures(out.events,&[88],&mut targets);if let Some((_,target))=targets.first() {let expected=self.text(*target);if expected!=name {self.effect(ScanEffect::Diagnostic{message:format!("back-reference mismatch: expected '{expected}' but got '{name}'"),offset_cp,len_cp,severity:Severity::Error});}}self.give_scope(targets);}
let _=(&name,offset_cp,len_cp); }
self.give_scope(scope_sites);
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:71,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["QuantifiedRefParser"]);}
self.depth-=1;out.diag=self.diag_rule(71,out.diag);out
}
fn r72_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e523_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:72,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@typeof'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(72,out.diag);out
}
fn r73_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e528_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:73,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["DottedIdentifierParser"]);}
self.depth-=1;out.diag=self.diag_rule(73,out.diag);out
}
fn r74_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e534_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:74,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["RuleBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(74,out.diag);out
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
out=self.literal::<false>(out.state,"@import",true,21,true,"@import");
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
out=self.literal::<false>(out.state,"from",true,63,true,"from");
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
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
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
out=self.literal::<false>(out.state,"grammar",true,64,true,"grammar");
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
out=self.literal::<false>(out.state,"{",true,78,false,"{");
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
out=self.r26_c(out.state);
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
out=self.literal::<false>(out.state,"}",true,80,false,"}");
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
out=self.literal::<false>(out.state,"@",true,13,false,"@");
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
out=self.literal::<false>(out.state,":",true,8,false,":");
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
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
out.events=self.event(Event::Capture {site:10,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'2'"]);}
out
}
fn b30_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e31_c(out.state);
out
}
fn e31_c(&mut self,state:State)->Step {
let mut out=self.b31_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'2'"]);}
out
}
fn b31_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
self.restore(mark);let mut child=self.e32_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e33_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e32_c(&mut self,state:State)->Step {
let mut out=self.b32_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b32_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r73_c(out.state);
out
}
fn e33_c(&mut self,state:State)->Step {
let mut out=self.b33_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,33,5);
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'2'"]);}
out
}
fn b33_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"2",true,7,true,"2");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'2'");}
out
}
fn e34_c(&mut self,state:State)->Step {
let mut out=self.b34_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b34_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e35_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e36_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e38_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e35_c(&mut self,state:State)->Step {
let mut out=self.b35_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,35,6);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b35_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,78,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e36_c(&mut self,state:State)->Step {
let mut out=self.b36_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'", "__CaptureSite"]);}
out
}
fn b36_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e37_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:11,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e37_c(&mut self,state:State)->Step {
let mut out=self.b37_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b37_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e38_c(&mut self,state:State)->Step {
let mut out=self.b38_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,38,6);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b38_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,80,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e39_c(&mut self,state:State)->Step {
let mut out=self.b39_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b39_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e40_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e41_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e42_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e40_c(&mut self,state:State)->Step {
let mut out=self.b40_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,40,7);
out.events=self.event(Event::Capture {site:12,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b40_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e41_c(&mut self,state:State)->Step {
let mut out=self.b41_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,41,7);
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b41_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,":",true,8,false,":");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"':'");}
out
}
fn e42_c(&mut self,state:State)->Step {
let mut out=self.b42_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,42,7);
out.events=self.event(Event::Capture {site:13,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b42_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e43_c(&mut self,state:State)->Step {
let mut out=self.b43_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b43_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e44_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e45_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e46_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e44_c(&mut self,state:State)->Step {
let mut out=self.b44_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,44,8);
} else {self.display_failures(state.consumed.max(state.matched),&["'token'"]);}
out
}
fn b44_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"token",true,75,true,"token");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'token'");}
out
}
fn e45_c(&mut self,state:State)->Step {
let mut out=self.b45_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,45,8);
out.events=self.event(Event::Capture {site:14,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b45_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e46_c(&mut self,state:State)->Step {
let mut out=self.b46_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'::='", "'='"]);}
out
}
fn b46_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e47_c(out.state);
out
}
fn e47_c(&mut self,state:State)->Step {
let mut out=self.b47_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'::='", "'='"]);}
out
}
fn b47_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,61))) {self.guard_e48_c(out.state.begin())} else {self.e48_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,58))) {self.guard_e51_c(out.state.begin())} else {self.e51_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e48_c(&mut self,state:State)->Step {
let mut out=self.b48_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
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
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b49_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e50_c(&mut self,state:State)->Step {
let mut out=self.b50_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:15,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'XID_IDENTIFIER'"]);}
out
}
fn b50_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r15_c(out.state);
out
}
fn e51_c(&mut self,state:State)->Step {
let mut out=self.b51_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'::='"]);}
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
let mut out=self.b52_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,52,8);
} else {self.display_failures(state.consumed.max(state.matched),&["'::='"]);}
out
}
fn b52_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"::=",true,9,false,"::=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'::='");}
out
}
fn e53_c(&mut self,state:State)->Step {
let mut out=self.b53_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:16,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
out
}
fn b53_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r9_c(out.state);
out
}
fn e54_c(&mut self,state:State)->Step {
let mut out=self.b54_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out.events=self.event(Event::Capture {site:17,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b55_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e56_c(&mut self,state:State)->Step {
let mut out=self.b56_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,56,9);
} else {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
out
}
fn b56_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,";",true,10,false,";");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"';'");}
out
}
fn e57_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:57,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b57_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e58_c(&mut self,state:State)->Step {
let mut out=self.b58_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:18,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b58_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r11_c(out.state);
out
}
fn e59_c(&mut self,state:State)->Step {
let mut out=self.b59_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b59_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e60_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e60_c(&mut self,state:State)->Step {
let mut out=self.b60_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b60_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e61_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e62_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e61_c(&mut self,state:State)->Step {
let mut out=self.b61_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,61,10);
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b61_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"|",true,79,false,"|");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'|'");}
out
}
fn e62_c(&mut self,state:State)->Step {
let mut out=self.b62_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:19,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b62_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r11_c(out.state);
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e64_c(&mut self,state:State)->Step {
let mut out=self.b64_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat", "__CaptureSite"]);}
out
}
fn b64_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e65_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:20,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if count<1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e65_c(&mut self,state:State)->Step {
let mut out=self.b65_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b65_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r12_c(out.state);
out
}
fn e66_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:66,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b66_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b66_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e67_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e68_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e67_c(&mut self,state:State)->Step {
let mut out=self.b67_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:21,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b67_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r14_c(out.state);
out
}
fn e68_c(&mut self,state:State)->Step {
let mut out=self.b68_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b68_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e69_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e69_c(&mut self,state:State)->Step {
let mut out=self.b69_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:22,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b69_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r13_c(out.state);
out
}
fn e70_c(&mut self,state:State)->Step {
let mut out=self.b70_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b70_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e71_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e71_c(&mut self,state:State)->Step {
let mut out=self.b71_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:23,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'*'", "'+'", "'?'", "'{'", "'}'"]);}
out
}
fn b71_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e72_c(out.state);
out
}
fn e72_c(&mut self,state:State)->Step {
let mut out=self.b72_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'*'", "'+'", "'?'", "'{'", "'}'"]);}
out
}
fn b72_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=self.e73_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e74_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e75_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,123))) {self.guard_e76_c(out.state.begin())} else {self.e76_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e73_c(&mut self,state:State)->Step {
let mut out=self.b73_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,73,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'?'"]);}
out
}
fn b73_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"?",true,12,false,"?");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'?'");}
out
}
fn e74_c(&mut self,state:State)->Step {
let mut out=self.b74_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,74,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'*'"]);}
out
}
fn b74_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"*",true,3,false,"*");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'*'");}
out
}
fn e75_c(&mut self,state:State)->Step {
let mut out=self.b75_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,75,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'+'"]);}
out
}
fn b75_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"+",true,4,false,"+");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'+'");}
out
}
fn e76_c(&mut self,state:State)->Step {
let mut out=self.b76_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
out
}
fn b76_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e77_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e78_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e79_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e84_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e77_c(&mut self,state:State)->Step {
let mut out=self.b77_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,77,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b77_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,78,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e78_c(&mut self,state:State)->Step {
let mut out=self.b78_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,78,13);
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser"]);}
out
}
fn b78_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_6::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e79_c(&mut self,state:State)->Step {
let mut out=self.b79_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
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
let mut out=self.b80_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b80_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e81_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e82_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e81_c(&mut self,state:State)->Step {
let mut out=self.b81_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,81,13);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b81_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e82_c(&mut self,state:State)->Step {
let mut out=self.b82_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b82_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e83_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e83_c(&mut self,state:State)->Step {
let mut out=self.b83_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,83,13);
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser"]);}
out
}
fn b83_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_6::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e84_c(&mut self,state:State)->Step {
let mut out=self.b84_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,84,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b84_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,80,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e85_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:85,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b85_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b85_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e86_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e86_c(&mut self,state:State)->Step {
let mut out=self.b86_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
out.events=self.event(Event::Capture {site:24,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "','", "'CAPTURE'", "'CHAR_RANGE'", "'NEGATION'", "'SAME_AS'", "'XID_IDENTIFIER'", "'['", "']'", "'{'", "'}'"]);}
out
}
fn b86_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e87_c(out.state);
out
}
fn e87_c(&mut self,state:State)->Step {
let mut out=self.b87_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "','", "'CAPTURE'", "'CHAR_RANGE'", "'NEGATION'", "'SAME_AS'", "'XID_IDENTIFIER'", "'['", "']'", "'{'", "'}'"]);}
out
}
fn b87_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=self.e88_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,40))) {self.guard_e89_c(out.state.begin())} else {self.e89_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,91))) {self.guard_e93_c(out.state.begin())} else {self.e93_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,123))) {self.guard_e97_c(out.state.begin())} else {self.e97_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,76|78))) {self.guard_e101_c(out.state.begin())} else {self.e101_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e109_c(out.state.begin())} else {self.e109_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,83))) {self.guard_e116_c(out.state.begin())} else {self.e116_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e121_c(out.state.begin())} else {self.e121_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,78))) {self.guard_e128_c(out.state.begin())} else {self.e128_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e133_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e134_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e88_c(&mut self,state:State)->Step {
let mut out=self.b88_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,88,14);
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser"]);}
out
}
fn b88_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e89_c(&mut self,state:State)->Step {
let mut out=self.b89_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e90_c(&mut self,state:State)->Step {
let mut out=self.b90_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,90,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b90_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e91_c(&mut self,state:State)->Step {
let mut out=self.b91_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b91_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e92_c(&mut self,state:State)->Step {
let mut out=self.b92_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,92,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b92_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e93_c(&mut self,state:State)->Step {
let mut out=self.b93_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'['", "']'"]);}
out
}
fn b93_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e94_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e95_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e96_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e94_c(&mut self,state:State)->Step {
let mut out=self.b94_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,94,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b94_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,59,false,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e95_c(&mut self,state:State)->Step {
let mut out=self.b95_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b95_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e96_c(&mut self,state:State)->Step {
let mut out=self.b96_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,96,14);
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b96_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,60,false,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e97_c(&mut self,state:State)->Step {
let mut out=self.b97_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
out
}
fn b97_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e98_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e99_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e100_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e98_c(&mut self,state:State)->Step {
let mut out=self.b98_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,98,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b98_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,78,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e99_c(&mut self,state:State)->Step {
let mut out=self.b99_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b99_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e100_c(&mut self,state:State)->Step {
let mut out=self.b100_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,100,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b100_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,80,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e101_c(&mut self,state:State)->Step {
let mut out=self.b101_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b101_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e102_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e106_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e107_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e108_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e102_c(&mut self,state:State)->Step {
let mut out=self.b102_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b102_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e103_c(out.state);
out
}
fn e103_c(&mut self,state:State)->Step {
let mut out=self.b103_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b103_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
self.restore(mark);let mut child=self.e104_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e105_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e104_c(&mut self,state:State)->Step {
let mut out=self.b104_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,104,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'"]);}
out
}
fn b104_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"LOOKAHEAD",true,52,true,"LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'LOOKAHEAD'");}
out
}
fn e105_c(&mut self,state:State)->Step {
let mut out=self.b105_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,105,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b105_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATIVE_LOOKAHEAD",true,54,true,"NEGATIVE_LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATIVE_LOOKAHEAD'");}
out
}
fn e106_c(&mut self,state:State)->Step {
let mut out=self.b106_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,106,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b106_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e107_c(&mut self,state:State)->Step {
let mut out=self.b107_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b107_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e108_c(&mut self,state:State)->Step {
let mut out=self.b108_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,108,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b108_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e109_c(&mut self,state:State)->Step {
let mut out=self.b109_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "','", "'CAPTURE'"]);}
out
}
fn b109_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e110_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e111_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e112_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e113_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e114_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e115_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e110_c(&mut self,state:State)->Step {
let mut out=self.b110_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,110,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'CAPTURE'"]);}
out
}
fn b110_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CAPTURE",true,41,true,"CAPTURE");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CAPTURE'");}
out
}
fn e111_c(&mut self,state:State)->Step {
let mut out=self.b111_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,111,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b111_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e112_c(&mut self,state:State)->Step {
let mut out=self.b112_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,112,14);
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser"]);}
out
}
fn b112_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e113_c(&mut self,state:State)->Step {
let mut out=self.b113_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,113,14);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b113_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
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
out=self.r10_c(out.state);
out
}
fn e115_c(&mut self,state:State)->Step {
let mut out=self.b115_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,115,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b115_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e116_c(&mut self,state:State)->Step {
let mut out=self.b116_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'SAME_AS'"]);}
out
}
fn b116_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e117_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e118_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e119_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e120_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e117_c(&mut self,state:State)->Step {
let mut out=self.b117_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,117,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'SAME_AS'"]);}
out
}
fn b117_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"SAME_AS",true,55,true,"SAME_AS");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'SAME_AS'");}
out
}
fn e118_c(&mut self,state:State)->Step {
let mut out=self.b118_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,118,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b118_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e119_c(&mut self,state:State)->Step {
let mut out=self.b119_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,119,14);
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser"]);}
out
}
fn b119_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e120_c(&mut self,state:State)->Step {
let mut out=self.b120_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,120,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b120_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e121_c(&mut self,state:State)->Step {
let mut out=self.b121_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "','", "'CHAR_RANGE'"]);}
out
}
fn b121_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e122_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e123_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e124_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e125_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e126_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e127_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e122_c(&mut self,state:State)->Step {
let mut out=self.b122_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,122,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'CHAR_RANGE'"]);}
out
}
fn b122_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CHAR_RANGE",true,45,true,"CHAR_RANGE");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CHAR_RANGE'");}
out
}
fn e123_c(&mut self,state:State)->Step {
let mut out=self.b123_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,123,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b123_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e124_c(&mut self,state:State)->Step {
let mut out=self.b124_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,124,14);
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser"]);}
out
}
fn b124_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e125_c(&mut self,state:State)->Step {
let mut out=self.b125_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,125,14);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b125_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e126_c(&mut self,state:State)->Step {
let mut out=self.b126_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,126,14);
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser"]);}
out
}
fn b126_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e127_c(&mut self,state:State)->Step {
let mut out=self.b127_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,127,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b127_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e128_c(&mut self,state:State)->Step {
let mut out=self.b128_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'NEGATION'"]);}
out
}
fn b128_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e129_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e130_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e131_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e132_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e129_c(&mut self,state:State)->Step {
let mut out=self.b129_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,129,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATION'"]);}
out
}
fn b129_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATION",true,53,true,"NEGATION");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATION'");}
out
}
fn e130_c(&mut self,state:State)->Step {
let mut out=self.b130_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,130,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b130_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e131_c(&mut self,state:State)->Step {
let mut out=self.b131_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,131,14);
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser"]);}
out
}
fn b131_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e132_c(&mut self,state:State)->Step {
let mut out=self.b132_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,132,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b132_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e133_c(&mut self,state:State)->Step {
let mut out=self.b133_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,133,14);
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'XID_IDENTIFIER'"]);}
out
}
fn b133_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"XID_IDENTIFIER",true,58,true,"XID_IDENTIFIER");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'XID_IDENTIFIER'");}
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
out=self.r73_c(out.state);
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
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,85))) {self.guard_e136_c(out.state.begin())} else {self.e136_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,78))) {self.guard_e137_c(out.state.begin())} else {self.e137_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,76))) {self.guard_e138_c(out.state.begin())} else {self.e138_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,78))) {self.guard_e139_c(out.state.begin())} else {self.e139_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e140_c(out.state.begin())} else {self.e140_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e141_c(out.state.begin())} else {self.e141_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,65))) {self.guard_e142_c(out.state.begin())} else {self.e142_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,65))) {self.guard_e143_c(out.state.begin())} else {self.e143_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e144_c(out.state.begin())} else {self.e144_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e145_c(out.state.begin())} else {self.e145_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e146_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e147_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
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
out=self.r16_c(out.state);
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
out=self.r17_c(out.state);
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
out=self.r18_c(out.state);
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
out=self.r19_c(out.state);
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
out=self.r20_c(out.state);
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
out=self.r21_c(out.state);
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
out=self.r22_c(out.state);
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
out=self.r23_c(out.state);
out
}
fn e144_c(&mut self,state:State)->Step {
let mut out=self.b144_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b144_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r24_c(out.state);
out
}
fn e145_c(&mut self,state:State)->Step {
let mut out=self.b145_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b145_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r25_c(out.state);
out
}
fn e146_c(&mut self,state:State)->Step {
let mut out=self.b146_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,146,15);
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'XID_IDENTIFIER'"]);}
out
}
fn b146_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"XID_IDENTIFIER",true,58,true,"XID_IDENTIFIER");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'XID_IDENTIFIER'");}
out
}
fn e147_c(&mut self,state:State)->Step {
let mut out=self.b147_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b147_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r73_c(out.state);
out
}
fn e148_c(&mut self,state:State)->Step {
let mut out=self.b148_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b148_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e149_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e150_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e151_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e152_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e149_c(&mut self,state:State)->Step {
let mut out=self.b149_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,149,16);
} else {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'"]);}
out
}
fn b149_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"UNTIL",true,57,true,"UNTIL");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'UNTIL'");}
out
}
fn e150_c(&mut self,state:State)->Step {
let mut out=self.b150_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,150,16);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b150_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e151_c(&mut self,state:State)->Step {
let mut out=self.b151_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,151,16);
out.events=self.event(Event::Capture {site:25,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b151_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e152_c(&mut self,state:State)->Step {
let mut out=self.b152_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,152,16);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b152_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e153_c(&mut self,state:State)->Step {
let mut out=self.b153_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b153_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e154_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e155_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e156_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e157_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e154_c(&mut self,state:State)->Step {
let mut out=self.b154_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,154,17);
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATION'"]);}
out
}
fn b154_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATION",true,53,true,"NEGATION");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATION'");}
out
}
fn e155_c(&mut self,state:State)->Step {
let mut out=self.b155_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,155,17);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b155_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e156_c(&mut self,state:State)->Step {
let mut out=self.b156_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,156,17);
out.events=self.event(Event::Capture {site:26,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b156_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e157_c(&mut self,state:State)->Step {
let mut out=self.b157_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,157,17);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b157_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e158_c(&mut self,state:State)->Step {
let mut out=self.b158_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b158_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e159_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
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
fn e159_c(&mut self,state:State)->Step {
let mut out=self.b159_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,159,18);
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'"]);}
out
}
fn b159_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"LOOKAHEAD",true,52,true,"LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'LOOKAHEAD'");}
out
}
fn e160_c(&mut self,state:State)->Step {
let mut out=self.b160_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,160,18);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b160_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e161_c(&mut self,state:State)->Step {
let mut out=self.b161_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,161,18);
out.events=self.event(Event::Capture {site:27,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b161_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e162_c(&mut self,state:State)->Step {
let mut out=self.b162_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,162,18);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b162_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e163_c(&mut self,state:State)->Step {
let mut out=self.b163_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e164_c(&mut self,state:State)->Step {
let mut out=self.b164_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,164,19);
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b164_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATIVE_LOOKAHEAD",true,54,true,"NEGATIVE_LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATIVE_LOOKAHEAD'");}
out
}
fn e165_c(&mut self,state:State)->Step {
let mut out=self.b165_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,165,19);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b165_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e166_c(&mut self,state:State)->Step {
let mut out=self.b166_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,166,19);
out.events=self.event(Event::Capture {site:28,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b166_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e167_c(&mut self,state:State)->Step {
let mut out=self.b167_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,167,19);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b167_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e168_c(&mut self,state:State)->Step {
let mut out=self.b168_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b168_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e169_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e170_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e171_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e172_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e173_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e174_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e169_c(&mut self,state:State)->Step {
let mut out=self.b169_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,169,20);
} else {self.display_failures(state.consumed.max(state.matched),&["'CHAR_RANGE'"]);}
out
}
fn b169_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CHAR_RANGE",true,45,true,"CHAR_RANGE");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CHAR_RANGE'");}
out
}
fn e170_c(&mut self,state:State)->Step {
let mut out=self.b170_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,170,20);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b170_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e171_c(&mut self,state:State)->Step {
let mut out=self.b171_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,171,20);
out.events=self.event(Event::Capture {site:29,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b171_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e172_c(&mut self,state:State)->Step {
let mut out=self.b172_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,172,20);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b172_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e173_c(&mut self,state:State)->Step {
let mut out=self.b173_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,173,20);
out.events=self.event(Event::Capture {site:30,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b173_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e174_c(&mut self,state:State)->Step {
let mut out=self.b174_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,174,20);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b174_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e175_c(&mut self,state:State)->Step {
let mut out=self.b175_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b175_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e176_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e177_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e178_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e179_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e176_c(&mut self,state:State)->Step {
let mut out=self.b176_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,176,21);
} else {self.display_failures(state.consumed.max(state.matched),&["'CI'"]);}
out
}
fn b176_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CI",true,46,true,"CI");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CI'");}
out
}
fn e177_c(&mut self,state:State)->Step {
let mut out=self.b177_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,177,21);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b177_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e178_c(&mut self,state:State)->Step {
let mut out=self.b178_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,178,21);
out.events=self.event(Event::Capture {site:31,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b178_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e179_c(&mut self,state:State)->Step {
let mut out=self.b179_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,179,21);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b179_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e180_c(&mut self,state:State)->Step {
let mut out=self.b180_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b180_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e181_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e181_c(&mut self,state:State)->Step {
let mut out=self.b181_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,181,22);
} else {self.display_failures(state.consumed.max(state.matched),&["'ADAPTER'"]);}
out
}
fn b181_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ADAPTER",true,39,true,"ADAPTER");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'ADAPTER'");}
out
}
fn e182_c(&mut self,state:State)->Step {
let mut out=self.b182_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,182,22);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b182_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e183_c(&mut self,state:State)->Step {
let mut out=self.b183_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,183,22);
out.events=self.event(Event::Capture {site:32,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b183_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e184_c(&mut self,state:State)->Step {
let mut out=self.b184_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,184,22);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b184_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e185_c(&mut self,state:State)->Step {
let mut out=self.b185_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,185,22);
} else {self.display_failures(state.consumed.max(state.matched),&["'version'"]);}
out
}
fn b185_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"version",true,77,true,"version");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'version'");}
out
}
fn e186_c(&mut self,state:State)->Step {
let mut out=self.b186_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,186,22);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b186_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e187_c(&mut self,state:State)->Step {
let mut out=self.b187_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,187,22);
out.events=self.event(Event::Capture {site:33,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser", "__CaptureSite"]);}
out
}
fn b187_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_6::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e188_c(&mut self,state:State)->Step {
let mut out=self.b188_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,188,22);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b188_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e189_c(&mut self,state:State)->Step {
let mut out=self.b189_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b189_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e190_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e190_c(&mut self,state:State)->Step {
let mut out=self.b190_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,190,23);
} else {self.display_failures(state.consumed.max(state.matched),&["'ANY'"]);}
out
}
fn b190_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ANY",true,40,true,"ANY");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'ANY'");}
out
}
fn e191_c(&mut self,state:State)->Step {
let mut out=self.b191_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b191_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e192_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e192_c(&mut self,state:State)->Step {
let mut out=self.b192_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,192,24);
} else {self.display_failures(state.consumed.max(state.matched),&["'EOF'"]);}
out
}
fn b192_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"EOF",true,49,true,"EOF");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'EOF'");}
out
}
fn e193_c(&mut self,state:State)->Step {
let mut out=self.b193_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b193_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e194_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e194_c(&mut self,state:State)->Step {
let mut out=self.b194_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,194,25);
} else {self.display_failures(state.consumed.max(state.matched),&["'EMPTY'"]);}
out
}
fn b194_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"EMPTY",true,48,true,"EMPTY");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'EMPTY'");}
out
}
fn e195_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:195,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b195_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b195_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e196_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e198_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e199_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e200_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e201_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e196_c(&mut self,state:State)->Step {
let mut out=self.b196_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat", "__CaptureSite"]);}
out
}
fn b196_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e197_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:34,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e197_c(&mut self,state:State)->Step {
let mut out=self.b197_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b197_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r27_c(out.state);
out
}
fn e198_c(&mut self,state:State)->Step {
let mut out=self.b198_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,198,26);
out.events=self.event(Event::Capture {site:35,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b198_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e199_c(&mut self,state:State)->Step {
let mut out=self.b199_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,199,26);
} else {self.display_failures(state.consumed.max(state.matched),&["'::='"]);}
out
}
fn b199_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"::=",true,9,false,"::=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'::='");}
out
}
fn e200_c(&mut self,state:State)->Step {
let mut out=self.b200_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:36,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b200_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e201_c(&mut self,state:State)->Step {
let mut out=self.b201_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,201,26);
} else {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
out
}
fn b201_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,";",true,10,false,";");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"';'");}
out
}
fn e202_c(&mut self,state:State)->Step {
let mut out=self.b202_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b202_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e203_c(out.state.begin())} else {self.e203_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
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
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e224_c(out.state.begin())} else {self.e224_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e225_c(out.state.begin())} else {self.e225_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e226_c(out.state.begin())} else {self.e226_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
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
out=self.r28_c(out.state);
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
out=self.r33_c(out.state);
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
out=self.r32_c(out.state);
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
let mut out=Step::yes(state);
out=self.r49_c(out.state);
out
}
fn e225_c(&mut self,state:State)->Step {
let mut out=self.b225_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b225_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r50_c(out.state);
out
}
fn e226_c(&mut self,state:State)->Step {
let mut out=self.b226_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b226_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r51_c(out.state);
out
}
fn e227_c(&mut self,state:State)->Step {
let mut out=self.b227_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b227_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e228_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e228_c(&mut self,state:State)->Step {
let mut out=self.b228_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,228,28);
} else {self.display_failures(state.consumed.max(state.matched),&["'@root'"]);}
out
}
fn b228_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@root",true,33,true,"@root");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@root'");}
out
}
fn e229_c(&mut self,state:State)->Step {
let mut out=self.b229_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
let child=self.e233_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e245_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e230_c(&mut self,state:State)->Step {
let mut out=self.b230_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,230,29);
} else {self.display_failures(state.consumed.max(state.matched),&["'@mapping'"]);}
out
}
fn b230_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@mapping",true,26,true,"@mapping");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@mapping'");}
out
}
fn e231_c(&mut self,state:State)->Step {
let mut out=self.b231_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,231,29);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b231_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e232_c(&mut self,state:State)->Step {
let mut out=self.b232_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,232,29);
out.events=self.event(Event::Capture {site:37,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["CLASS_NAMEParser", "__CaptureSite"]);}
out
}
fn b232_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_5::<false>(out.state,"CLASS_NAME","CLASS_NAMEParser");
out
}
fn e233_c(&mut self,state:State)->Step {
let mut out=self.b233_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'['", "']'", "'params'"]);}
out
}
fn b233_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e234_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e234_c(&mut self,state:State)->Step {
let mut out=self.b234_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'['", "']'", "'params'"]);}
out
}
fn b234_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e235_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e236_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e237_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e238_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e239_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e240_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e244_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e235_c(&mut self,state:State)->Step {
let mut out=self.b235_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,235,29);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b235_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e236_c(&mut self,state:State)->Step {
let mut out=self.b236_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,236,29);
} else {self.display_failures(state.consumed.max(state.matched),&["'params'"]);}
out
}
fn b236_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"params",true,70,true,"params");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'params'");}
out
}
fn e237_c(&mut self,state:State)->Step {
let mut out=self.b237_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,237,29);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b237_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e238_c(&mut self,state:State)->Step {
let mut out=self.b238_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,238,29);
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b238_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,59,false,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e239_c(&mut self,state:State)->Step {
let mut out=self.b239_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,239,29);
out.events=self.event(Event::Capture {site:38,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b239_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e240_c(&mut self,state:State)->Step {
let mut out=self.b240_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b240_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e241_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e241_c(&mut self,state:State)->Step {
let mut out=self.b241_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b241_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e242_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e243_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e242_c(&mut self,state:State)->Step {
let mut out=self.b242_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,242,29);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b242_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e243_c(&mut self,state:State)->Step {
let mut out=self.b243_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,243,29);
out.events=self.event(Event::Capture {site:39,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b243_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e244_c(&mut self,state:State)->Step {
let mut out=self.b244_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,244,29);
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b244_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,60,false,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e245_c(&mut self,state:State)->Step {
let mut out=self.b245_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,245,29);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b245_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e246_c(&mut self,state:State)->Step {
let mut out=self.b246_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b246_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e247_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e248_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e249_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e250_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e254_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e247_c(&mut self,state:State)->Step {
let mut out=self.b247_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,247,30);
} else {self.display_failures(state.consumed.max(state.matched),&["'@eval'"]);}
out
}
fn b247_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@eval",true,20,true,"@eval");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@eval'");}
out
}
fn e248_c(&mut self,state:State)->Step {
let mut out=self.b248_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,248,30);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b248_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e249_c(&mut self,state:State)->Step {
let mut out=self.b249_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:40,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b249_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e250_c(&mut self,state:State)->Step {
let mut out=self.b250_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b250_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e251_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e251_c(&mut self,state:State)->Step {
let mut out=self.b251_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
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
let mut out=self.b252_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,252,30);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b252_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e253_c(&mut self,state:State)->Step {
let mut out=self.b253_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:41,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b253_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e254_c(&mut self,state:State)->Step {
let mut out=self.b254_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,254,30);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b254_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e255_c(&mut self,state:State)->Step {
let mut out=self.b255_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e256_c(&mut self,state:State)->Step {
let mut out=self.b256_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,256,31);
} else {self.display_failures(state.consumed.max(state.matched),&["'@whitespace'"]);}
out
}
fn b256_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@whitespace",true,38,true,"@whitespace");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@whitespace'");}
out
}
fn e257_c(&mut self,state:State)->Step {
let mut out=self.b257_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
out
}
fn b257_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e258_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e258_c(&mut self,state:State)->Step {
let mut out=self.b258_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
out
}
fn b258_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e259_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e260_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e261_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e259_c(&mut self,state:State)->Step {
let mut out=self.b259_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,259,31);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b259_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e260_c(&mut self,state:State)->Step {
let mut out=self.b260_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,260,31);
out.events=self.event(Event::Capture {site:42,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b260_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e261_c(&mut self,state:State)->Step {
let mut out=self.b261_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,261,31);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b261_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e262_c(&mut self,state:State)->Step {
let mut out=self.b262_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b262_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e263_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e264_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e265_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e266_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e267_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e268_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e275_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e276_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e290_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e263_c(&mut self,state:State)->Step {
let mut out=self.b263_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,263,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'@lexicalContext'"]);}
out
}
fn b263_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@lexicalContext",true,24,true,"@lexicalContext");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@lexicalContext'");}
out
}
fn e264_c(&mut self,state:State)->Step {
let mut out=self.b264_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,264,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b264_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e265_c(&mut self,state:State)->Step {
let mut out=self.b265_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,265,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'tokens'"]);}
out
}
fn b265_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"tokens",true,76,true,"tokens");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'tokens'");}
out
}
fn e266_c(&mut self,state:State)->Step {
let mut out=self.b266_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,266,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b266_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e267_c(&mut self,state:State)->Step {
let mut out=self.b267_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,267,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b267_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,59,false,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e268_c(&mut self,state:State)->Step {
let mut out=self.b268_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
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
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e270_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e271_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e270_c(&mut self,state:State)->Step {
let mut out=self.b270_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,270,32);
out.events=self.event(Event::Capture {site:43,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b270_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e271_c(&mut self,state:State)->Step {
let mut out=self.b271_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b271_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e272_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e272_c(&mut self,state:State)->Step {
let mut out=self.b272_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b272_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e273_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e274_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e273_c(&mut self,state:State)->Step {
let mut out=self.b273_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,273,32);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b273_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e274_c(&mut self,state:State)->Step {
let mut out=self.b274_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,274,32);
out.events=self.event(Event::Capture {site:44,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b274_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e275_c(&mut self,state:State)->Step {
let mut out=self.b275_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,275,32);
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b275_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,60,false,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e276_c(&mut self,state:State)->Step {
let mut out=self.b276_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'['", "']'", "'literals'"]);}
out
}
fn b276_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e277_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e277_c(&mut self,state:State)->Step {
let mut out=self.b277_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'['", "']'", "'literals'"]);}
out
}
fn b277_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e278_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e279_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e280_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e281_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e282_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e289_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e278_c(&mut self,state:State)->Step {
let mut out=self.b278_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,278,32);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b278_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e279_c(&mut self,state:State)->Step {
let mut out=self.b279_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,279,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'literals'"]);}
out
}
fn b279_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"literals",true,67,true,"literals");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'literals'");}
out
}
fn e280_c(&mut self,state:State)->Step {
let mut out=self.b280_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,280,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b280_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e281_c(&mut self,state:State)->Step {
let mut out=self.b281_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,281,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b281_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,59,false,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e282_c(&mut self,state:State)->Step {
let mut out=self.b282_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b282_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e283_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e283_c(&mut self,state:State)->Step {
let mut out=self.b283_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b283_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e284_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e285_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e284_c(&mut self,state:State)->Step {
let mut out=self.b284_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,284,32);
out.events=self.event(Event::Capture {site:45,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b284_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e285_c(&mut self,state:State)->Step {
let mut out=self.b285_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b285_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e286_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e286_c(&mut self,state:State)->Step {
let mut out=self.b286_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b286_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e287_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e288_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e287_c(&mut self,state:State)->Step {
let mut out=self.b287_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,287,32);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b287_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e288_c(&mut self,state:State)->Step {
let mut out=self.b288_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,288,32);
out.events=self.event(Event::Capture {site:46,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b288_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e289_c(&mut self,state:State)->Step {
let mut out=self.b289_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,289,32);
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b289_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,60,false,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e290_c(&mut self,state:State)->Step {
let mut out=self.b290_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,290,32);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b290_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e291_c(&mut self,state:State)->Step {
let mut out=self.b291_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b291_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e292_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e293_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e294_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e295_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e296_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e297_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e292_c(&mut self,state:State)->Step {
let mut out=self.b292_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,292,33);
} else {self.display_failures(state.consumed.max(state.matched),&["'@interleave'"]);}
out
}
fn b292_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@interleave",true,22,true,"@interleave");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@interleave'");}
out
}
fn e293_c(&mut self,state:State)->Step {
let mut out=self.b293_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,293,33);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b293_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e294_c(&mut self,state:State)->Step {
let mut out=self.b294_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,294,33);
} else {self.display_failures(state.consumed.max(state.matched),&["'profile'"]);}
out
}
fn b294_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"profile",true,71,true,"profile");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'profile'");}
out
}
fn e295_c(&mut self,state:State)->Step {
let mut out=self.b295_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,295,33);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b295_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e296_c(&mut self,state:State)->Step {
let mut out=self.b296_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,296,33);
out.events=self.event(Event::Capture {site:47,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b296_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e297_c(&mut self,state:State)->Step {
let mut out=self.b297_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,297,33);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b297_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e298_c(&mut self,state:State)->Step {
let mut out=self.b298_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b298_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e299_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e300_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e301_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
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
fn e299_c(&mut self,state:State)->Step {
let mut out=self.b299_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,299,34);
} else {self.display_failures(state.consumed.max(state.matched),&["'@backref'"]);}
out
}
fn b299_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@backref",true,14,true,"@backref");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@backref'");}
out
}
fn e300_c(&mut self,state:State)->Step {
let mut out=self.b300_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,300,34);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b300_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e301_c(&mut self,state:State)->Step {
let mut out=self.b301_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,301,34);
} else {self.display_failures(state.consumed.max(state.matched),&["'name'"]);}
out
}
fn b301_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"name",true,69,true,"name");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'name'");}
out
}
fn e302_c(&mut self,state:State)->Step {
let mut out=self.b302_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,302,34);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b302_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e303_c(&mut self,state:State)->Step {
let mut out=self.b303_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,303,34);
out.events=self.event(Event::Capture {site:48,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b303_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e304_c(&mut self,state:State)->Step {
let mut out=self.b304_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,304,34);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b304_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e305_c(&mut self,state:State)->Step {
let mut out=self.b305_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
let child=self.e309_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e310_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e311_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e306_c(&mut self,state:State)->Step {
let mut out=self.b306_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,306,35);
} else {self.display_failures(state.consumed.max(state.matched),&["'@scopeTree'"]);}
out
}
fn b306_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@scopeTree",true,34,true,"@scopeTree");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@scopeTree'");}
out
}
fn e307_c(&mut self,state:State)->Step {
let mut out=self.b307_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,307,35);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b307_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e308_c(&mut self,state:State)->Step {
let mut out=self.b308_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,308,35);
} else {self.display_failures(state.consumed.max(state.matched),&["'mode'"]);}
out
}
fn b308_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"mode",true,68,true,"mode");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'mode'");}
out
}
fn e309_c(&mut self,state:State)->Step {
let mut out=self.b309_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,309,35);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b309_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e310_c(&mut self,state:State)->Step {
let mut out=self.b310_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,310,35);
out.events=self.event(Event::Capture {site:49,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b310_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e311_c(&mut self,state:State)->Step {
let mut out=self.b311_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,311,35);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b311_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e312_c(&mut self,state:State)->Step {
let mut out=self.b312_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b312_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e313_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e314_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e315_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e316_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e317_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e318_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e324_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e313_c(&mut self,state:State)->Step {
let mut out=self.b313_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,313,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'@declares'"]);}
out
}
fn b313_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@declares",true,17,true,"@declares");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@declares'");}
out
}
fn e314_c(&mut self,state:State)->Step {
let mut out=self.b314_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,314,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b314_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e315_c(&mut self,state:State)->Step {
let mut out=self.b315_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,315,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'symbol'"]);}
out
}
fn b315_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"symbol",true,73,true,"symbol");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'symbol'");}
out
}
fn e316_c(&mut self,state:State)->Step {
let mut out=self.b316_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,316,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b316_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e317_c(&mut self,state:State)->Step {
let mut out=self.b317_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,317,36);
out.events=self.event(Event::Capture {site:50,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b317_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e318_c(&mut self,state:State)->Step {
let mut out=self.b318_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'description'"]);}
out
}
fn b318_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e319_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e319_c(&mut self,state:State)->Step {
let mut out=self.b319_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'description'"]);}
out
}
fn b319_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e320_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e321_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e322_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e323_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e320_c(&mut self,state:State)->Step {
let mut out=self.b320_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,320,36);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b320_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e321_c(&mut self,state:State)->Step {
let mut out=self.b321_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,321,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'description'"]);}
out
}
fn b321_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"description",true,62,true,"description");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'description'");}
out
}
fn e322_c(&mut self,state:State)->Step {
let mut out=self.b322_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,322,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b322_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e323_c(&mut self,state:State)->Step {
let mut out=self.b323_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,323,36);
out.events=self.event(Event::Capture {site:51,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b323_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e324_c(&mut self,state:State)->Step {
let mut out=self.b324_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,324,36);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b324_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e325_c(&mut self,state:State)->Step {
let mut out=self.b325_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
let child=self.e329_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e330_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e331_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e326_c(&mut self,state:State)->Step {
let mut out=self.b326_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,326,37);
} else {self.display_failures(state.consumed.max(state.matched),&["'@catalog'"]);}
out
}
fn b326_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@catalog",true,15,true,"@catalog");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@catalog'");}
out
}
fn e327_c(&mut self,state:State)->Step {
let mut out=self.b327_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,327,37);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b327_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e328_c(&mut self,state:State)->Step {
let mut out=self.b328_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,328,37);
} else {self.display_failures(state.consumed.max(state.matched),&["'context'"]);}
out
}
fn b328_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"context",true,61,true,"context");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'context'");}
out
}
fn e329_c(&mut self,state:State)->Step {
let mut out=self.b329_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,329,37);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b329_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e330_c(&mut self,state:State)->Step {
let mut out=self.b330_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,330,37);
out.events=self.event(Event::Capture {site:52,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b330_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e331_c(&mut self,state:State)->Step {
let mut out=self.b331_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,331,37);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b331_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e332_c(&mut self,state:State)->Step {
let mut out=self.b332_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b332_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e333_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e333_c(&mut self,state:State)->Step {
let mut out=self.b333_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,333,38);
} else {self.display_failures(state.consumed.max(state.matched),&["'@leftAssoc'"]);}
out
}
fn b333_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@leftAssoc",true,23,true,"@leftAssoc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@leftAssoc'");}
out
}
fn e334_c(&mut self,state:State)->Step {
let mut out=self.b334_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b334_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e335_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e335_c(&mut self,state:State)->Step {
let mut out=self.b335_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,335,39);
} else {self.display_failures(state.consumed.max(state.matched),&["'@rightAssoc'"]);}
out
}
fn b335_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@rightAssoc",true,32,true,"@rightAssoc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@rightAssoc'");}
out
}
fn e336_c(&mut self,state:State)->Step {
let mut out=self.b336_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b336_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e337_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e337_c(&mut self,state:State)->Step {
let mut out=self.b337_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,337,40);
} else {self.display_failures(state.consumed.max(state.matched),&["'@longestChoice'"]);}
out
}
fn b337_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@longestChoice",true,25,true,"@longestChoice");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@longestChoice'");}
out
}
fn e338_c(&mut self,state:State)->Step {
let mut out=self.b338_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b338_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e339_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e339_c(&mut self,state:State)->Step {
let mut out=self.b339_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,339,41);
} else {self.display_failures(state.consumed.max(state.matched),&["'@uniqueLongestChoice'"]);}
out
}
fn b339_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@uniqueLongestChoice",true,37,true,"@uniqueLongestChoice");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@uniqueLongestChoice'");}
out
}
fn e340_c(&mut self,state:State)->Step {
let mut out=self.b340_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b340_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e341_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e342_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e343_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e344_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e345_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e346_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e347_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e348_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e349_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e350_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e351_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e352_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e353_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e354_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e355_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e356_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e357_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e358_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e341_c(&mut self,state:State)->Step {
let mut out=self.b341_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,341,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'@namePredicate'"]);}
out
}
fn b341_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@namePredicate",true,27,true,"@namePredicate");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@namePredicate'");}
out
}
fn e342_c(&mut self,state:State)->Step {
let mut out=self.b342_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,342,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b342_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e343_c(&mut self,state:State)->Step {
let mut out=self.b343_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,343,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'snapshot'"]);}
out
}
fn b343_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"snapshot",true,72,true,"snapshot");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'snapshot'");}
out
}
fn e344_c(&mut self,state:State)->Step {
let mut out=self.b344_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,344,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b344_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e345_c(&mut self,state:State)->Step {
let mut out=self.b345_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,345,42);
out.events=self.event(Event::Capture {site:53,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b345_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e346_c(&mut self,state:State)->Step {
let mut out=self.b346_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,346,42);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b346_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e347_c(&mut self,state:State)->Step {
let mut out=self.b347_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,347,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'version'"]);}
out
}
fn b347_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"version",true,77,true,"version");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'version'");}
out
}
fn e348_c(&mut self,state:State)->Step {
let mut out=self.b348_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,348,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b348_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e349_c(&mut self,state:State)->Step {
let mut out=self.b349_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,349,42);
out.events=self.event(Event::Capture {site:54,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b349_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e350_c(&mut self,state:State)->Step {
let mut out=self.b350_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,350,42);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b350_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e351_c(&mut self,state:State)->Step {
let mut out=self.b351_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,351,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'name'"]);}
out
}
fn b351_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"name",true,69,true,"name");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'name'");}
out
}
fn e352_c(&mut self,state:State)->Step {
let mut out=self.b352_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,352,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b352_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e353_c(&mut self,state:State)->Step {
let mut out=self.b353_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,353,42);
out.events=self.event(Event::Capture {site:55,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b353_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e354_c(&mut self,state:State)->Step {
let mut out=self.b354_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,354,42);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b354_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e355_c(&mut self,state:State)->Step {
let mut out=self.b355_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,355,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'kind'"]);}
out
}
fn b355_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"kind",true,65,true,"kind");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'kind'");}
out
}
fn e356_c(&mut self,state:State)->Step {
let mut out=self.b356_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,356,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b356_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e357_c(&mut self,state:State)->Step {
let mut out=self.b357_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,357,42);
out.events=self.event(Event::Capture {site:56,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b357_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e358_c(&mut self,state:State)->Step {
let mut out=self.b358_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,358,42);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b358_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e359_c(&mut self,state:State)->Step {
let mut out=self.b359_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b359_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e360_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e361_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e362_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e363_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e360_c(&mut self,state:State)->Step {
let mut out=self.b360_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,360,43);
} else {self.display_failures(state.consumed.max(state.matched),&["'@predicate'"]);}
out
}
fn b360_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@predicate",true,29,true,"@predicate");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@predicate'");}
out
}
fn e361_c(&mut self,state:State)->Step {
let mut out=self.b361_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,361,43);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b361_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e362_c(&mut self,state:State)->Step {
let mut out=self.b362_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,362,43);
out.events=self.event(Event::Capture {site:57,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b362_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e363_c(&mut self,state:State)->Step {
let mut out=self.b363_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,363,43);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b363_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e364_c(&mut self,state:State)->Step {
let mut out=self.b364_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b364_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e365_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e365_c(&mut self,state:State)->Step {
let mut out=self.b365_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,365,44);
} else {self.display_failures(state.consumed.max(state.matched),&["'@predictiveChoice'"]);}
out
}
fn b365_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@predictiveChoice",true,30,true,"@predictiveChoice");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@predictiveChoice'");}
out
}
fn e366_c(&mut self,state:State)->Step {
let mut out=self.b366_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b366_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e367_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e368_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e369_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e370_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e371_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e372_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e367_c(&mut self,state:State)->Step {
let mut out=self.b367_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,367,45);
} else {self.display_failures(state.consumed.max(state.matched),&["'@precedence'"]);}
out
}
fn b367_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@precedence",true,28,true,"@precedence");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@precedence'");}
out
}
fn e368_c(&mut self,state:State)->Step {
let mut out=self.b368_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,368,45);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b368_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e369_c(&mut self,state:State)->Step {
let mut out=self.b369_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,369,45);
} else {self.display_failures(state.consumed.max(state.matched),&["'level'"]);}
out
}
fn b369_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"level",true,66,true,"level");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'level'");}
out
}
fn e370_c(&mut self,state:State)->Step {
let mut out=self.b370_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,370,45);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b370_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e371_c(&mut self,state:State)->Step {
let mut out=self.b371_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,371,45);
out.events=self.event(Event::Capture {site:58,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser", "__CaptureSite"]);}
out
}
fn b371_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_6::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e372_c(&mut self,state:State)->Step {
let mut out=self.b372_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,372,45);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b372_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e373_c(&mut self,state:State)->Step {
let mut out=self.b373_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b373_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e374_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e375_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e376_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e377_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e374_c(&mut self,state:State)->Step {
let mut out=self.b374_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,374,46);
} else {self.display_failures(state.consumed.max(state.matched),&["'@doc'"]);}
out
}
fn b374_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@doc",true,18,true,"@doc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@doc'");}
out
}
fn e375_c(&mut self,state:State)->Step {
let mut out=self.b375_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,375,46);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b375_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e376_c(&mut self,state:State)->Step {
let mut out=self.b376_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,376,46);
out.events=self.event(Event::Capture {site:59,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b376_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e377_c(&mut self,state:State)->Step {
let mut out=self.b377_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,377,46);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b377_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e378_c(&mut self,state:State)->Step {
let mut out=self.b378_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b378_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e379_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e380_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e381_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e388_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e379_c(&mut self,state:State)->Step {
let mut out=self.b379_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,379,47);
} else {self.display_failures(state.consumed.max(state.matched),&["'@recovery'"]);}
out
}
fn b379_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@recovery",true,31,true,"@recovery");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@recovery'");}
out
}
fn e380_c(&mut self,state:State)->Step {
let mut out=self.b380_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,380,47);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b380_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e381_c(&mut self,state:State)->Step {
let mut out=self.b381_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
out
}
fn b381_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e382_c(out.state);
out
}
fn e382_c(&mut self,state:State)->Step {
let mut out=self.b382_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
out
}
fn b382_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,115))) {self.guard_e383_c(out.state.begin())} else {self.e383_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e387_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e383_c(&mut self,state:State)->Step {
let mut out=self.b383_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
out
}
fn b383_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e384_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e385_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e386_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e384_c(&mut self,state:State)->Step {
let mut out=self.b384_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,384,47);
} else {self.display_failures(state.consumed.max(state.matched),&["'sync'"]);}
out
}
fn b384_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"sync",true,74,true,"sync");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'sync'");}
out
}
fn e385_c(&mut self,state:State)->Step {
let mut out=self.b385_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,385,47);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b385_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,11,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e386_c(&mut self,state:State)->Step {
let mut out=self.b386_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,386,47);
out.events=self.event(Event::Capture {site:60,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b386_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e387_c(&mut self,state:State)->Step {
let mut out=self.b387_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,387,47);
out.events=self.event(Event::Capture {site:61,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b387_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e388_c(&mut self,state:State)->Step {
let mut out=self.b388_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,388,47);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b388_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e389_c(&mut self,state:State)->Step {
let mut out=self.b389_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b389_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e390_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e390_c(&mut self,state:State)->Step {
let mut out=self.b390_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,390,48);
} else {self.display_failures(state.consumed.max(state.matched),&["'@skip'"]);}
out
}
fn b390_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@skip",true,35,true,"@skip");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@skip'");}
out
}
fn e391_c(&mut self,state:State)->Step {
let mut out=self.b391_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b391_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e392_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e393_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e394_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e395_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e399_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e392_c(&mut self,state:State)->Step {
let mut out=self.b392_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,392,49);
} else {self.display_failures(state.consumed.max(state.matched),&["'@commonField'"]);}
out
}
fn b392_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@commonField",true,16,true,"@commonField");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@commonField'");}
out
}
fn e393_c(&mut self,state:State)->Step {
let mut out=self.b393_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,393,49);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b393_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e394_c(&mut self,state:State)->Step {
let mut out=self.b394_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,394,49);
out.events=self.event(Event::Capture {site:62,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b394_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e395_c(&mut self,state:State)->Step {
let mut out=self.b395_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b395_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e396_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e396_c(&mut self,state:State)->Step {
let mut out=self.b396_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b396_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e397_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e398_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e397_c(&mut self,state:State)->Step {
let mut out=self.b397_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,397,49);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b397_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e398_c(&mut self,state:State)->Step {
let mut out=self.b398_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,398,49);
out.events=self.event(Event::Capture {site:63,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b398_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e399_c(&mut self,state:State)->Step {
let mut out=self.b399_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,399,49);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b399_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e400_c(&mut self,state:State)->Step {
let mut out=self.b400_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b400_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e401_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e401_c(&mut self,state:State)->Step {
let mut out=self.b401_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,401,50);
} else {self.display_failures(state.consumed.max(state.matched),&["'@enum'"]);}
out
}
fn b401_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@enum",true,19,true,"@enum");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@enum'");}
out
}
fn e402_c(&mut self,state:State)->Step {
let mut out=self.b402_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b402_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e403_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e404_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e403_c(&mut self,state:State)->Step {
let mut out=self.b403_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,403,51);
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b403_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,13,false,"@");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@'");}
out
}
fn e404_c(&mut self,state:State)->Step {
let mut out=self.b404_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,404,51);
out.events=self.event(Event::Capture {site:64,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b404_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e405_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:405,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b405_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e406_c(&mut self,state:State)->Step {
let mut out=self.b406_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:65,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b406_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r53_c(out.state);
out
}
fn e407_c(&mut self,state:State)->Step {
let mut out=self.b407_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b407_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e408_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e408_c(&mut self,state:State)->Step {
let mut out=self.b408_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b408_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e409_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e410_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e409_c(&mut self,state:State)->Step {
let mut out=self.b409_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,409,52);
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b409_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"|",true,79,false,"|");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'|'");}
out
}
fn e410_c(&mut self,state:State)->Step {
let mut out=self.b410_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:66,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b410_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r53_c(out.state);
out
}
fn e411_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:411,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b411_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b411_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e412_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e412_c(&mut self,state:State)->Step {
let mut out=self.b412_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat", "__CaptureSite"]);}
out
}
fn b412_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e413_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:67,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if count<1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e413_c(&mut self,state:State)->Step {
let mut out=self.b413_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b413_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r54_c(out.state);
out
}
fn e414_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:414,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b414_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b414_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e415_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e417_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e418_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e420_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e422_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e424_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e415_c(&mut self,state:State)->Step {
let mut out=self.b415_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b415_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e416_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e416_c(&mut self,state:State)->Step {
let mut out=self.b416_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:68,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'@typeof'"]);}
out
}
fn b416_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r72_c(out.state);
out
}
fn e417_c(&mut self,state:State)->Step {
let mut out=self.b417_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:69,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b417_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r59_c(out.state);
out
}
fn e418_c(&mut self,state:State)->Step {
let mut out=self.b418_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'*'", "'+'", "'?'"]);}
out
}
fn b418_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e419_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e419_c(&mut self,state:State)->Step {
let mut out=self.b419_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b419_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r55_c(out.state);
out
}
fn e420_c(&mut self,state:State)->Step {
let mut out=self.b420_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
out
}
fn b420_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e421_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e421_c(&mut self,state:State)->Step {
let mut out=self.b421_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b421_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r57_c(out.state);
out
}
fn e422_c(&mut self,state:State)->Step {
let mut out=self.b422_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
out
}
fn b422_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e423_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e423_c(&mut self,state:State)->Step {
let mut out=self.b423_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b423_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r56_c(out.state);
out
}
fn e424_c(&mut self,state:State)->Step {
let mut out=self.b424_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b424_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e425_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e425_c(&mut self,state:State)->Step {
let mut out=self.b425_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b425_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e426_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e427_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e426_c(&mut self,state:State)->Step {
let mut out=self.b426_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,426,54);
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b426_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,13,false,"@");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@'");}
out
}
fn e427_c(&mut self,state:State)->Step {
let mut out=self.b427_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,427,54);
out.events=self.event(Event::Capture {site:70,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b427_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e428_c(&mut self,state:State)->Step {
let mut out=self.b428_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b428_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
if !out.state.invert {let predicted=match self.input.cp_at(out.state.begin().position::<false>()).map(|(c,_)|c) {
Some('+')=>0usize,
Some('?')=>1usize,
Some('*')=>2usize,
_=>usize::MAX,};let candidate=match predicted {
0=>Some(self.e429_c(out.state.begin())),
1=>Some(self.e430_c(out.state.begin())),
2=>Some(self.e431_c(out.state.begin())),
_=>None,};
if !DIAG && self.options.predict && predicted==usize::MAX {self.restore(mark);out.ok=false;out.diag=diagnostic;break 'choice out;}
if let Some(mut child)=candidate {if child.ok {
if predicted>0 {let d=self.diag_fail(out.state.begin().position::<false>(),"+");diagnostic=self.diag_join(diagnostic,d);}
if predicted>1 {let d=self.diag_fail(out.state.begin().position::<false>(),"?");diagnostic=self.diag_join(diagnostic,d);}
if predicted>2 {let d=self.diag_fail(out.state.begin().position::<false>(),"*");diagnostic=self.diag_join(diagnostic,d);}
child.state=out.state.commit(child.state);child.diag=self.diag_join(diagnostic,child.diag);break 'choice child;}}self.restore(mark); }
self.restore(mark);let mut child=self.e429_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e430_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e431_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e429_c(&mut self,state:State)->Step {
let mut out=self.b429_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,429,55);
} else {self.display_failures(state.consumed.max(state.matched),&["'+'"]);}
out
}
fn b429_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"+",true,4,false,"+");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'+'");}
out
}
fn e430_c(&mut self,state:State)->Step {
let mut out=self.b430_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,430,55);
} else {self.display_failures(state.consumed.max(state.matched),&["'?'"]);}
out
}
fn b430_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"?",true,12,false,"?");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'?'");}
out
}
fn e431_c(&mut self,state:State)->Step {
let mut out=self.b431_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,431,55);
} else {self.display_failures(state.consumed.max(state.matched),&["'*'"]);}
out
}
fn b431_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"*",true,3,false,"*");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'*'");}
out
}
fn e432_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:432,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b432_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b432_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e433_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e434_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e433_c(&mut self,state:State)->Step {
let mut out=self.b433_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,433,56);
} else {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
out
}
fn b433_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"%",true,0,false,"%");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'%'");}
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
out=self.r59_c(out.state);
out
}
fn e435_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:435,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b435_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b435_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e436_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e437_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e438_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e443_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e436_c(&mut self,state:State)->Step {
let mut out=self.b436_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,436,57);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b436_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,78,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e437_c(&mut self,state:State)->Step {
let mut out=self.b437_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b437_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e438_c(&mut self,state:State)->Step {
let mut out=self.b438_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b438_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e439_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e439_c(&mut self,state:State)->Step {
let mut out=self.b439_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b439_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e440_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e441_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e440_c(&mut self,state:State)->Step {
let mut out=self.b440_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,440,57);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b440_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e441_c(&mut self,state:State)->Step {
let mut out=self.b441_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b441_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e442_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e442_c(&mut self,state:State)->Step {
let mut out=self.b442_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b442_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e443_c(&mut self,state:State)->Step {
let mut out=self.b443_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,443,57);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b443_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,80,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e444_c(&mut self,state:State)->Step {
let mut out=self.b444_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b444_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e445_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e445_c(&mut self,state:State)->Step {
let mut out=self.b445_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,445,58);
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser"]);}
out
}
fn b445_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_6::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e446_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:446,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b446_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b446_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e447_c(out.state.begin())} else {self.e447_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,83))) {self.guard_e448_c(out.state.begin())} else {self.e448_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e449_c(out.state.begin())} else {self.e449_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e450_c(out.state.begin())} else {self.e450_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,76))) {self.guard_e451_c(out.state.begin())} else {self.e451_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e452_c(out.state.begin())} else {self.e452_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,40))) {self.guard_e453_c(out.state.begin())} else {self.e453_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,91))) {self.guard_e454_c(out.state.begin())} else {self.e454_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,123))) {self.guard_e455_c(out.state.begin())} else {self.e455_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e456_c(out.state.begin())} else {self.e456_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e457_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e458_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e447_c(&mut self,state:State)->Step {
let mut out=self.b447_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b447_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r60_c(out.state);
out
}
fn e448_c(&mut self,state:State)->Step {
let mut out=self.b448_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b448_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r61_c(out.state);
out
}
fn e449_c(&mut self,state:State)->Step {
let mut out=self.b449_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b449_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r62_c(out.state);
out
}
fn e450_c(&mut self,state:State)->Step {
let mut out=self.b450_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b450_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r63_c(out.state);
out
}
fn e451_c(&mut self,state:State)->Step {
let mut out=self.b451_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b451_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r64_c(out.state);
out
}
fn e452_c(&mut self,state:State)->Step {
let mut out=self.b452_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b452_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r65_c(out.state);
out
}
fn e453_c(&mut self,state:State)->Step {
let mut out=self.b453_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b453_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r66_c(out.state);
out
}
fn e454_c(&mut self,state:State)->Step {
let mut out=self.b454_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b454_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r67_c(out.state);
out
}
fn e455_c(&mut self,state:State)->Step {
let mut out=self.b455_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b455_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r68_c(out.state);
out
}
fn e456_c(&mut self,state:State)->Step {
let mut out=self.b456_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b456_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r69_c(out.state);
out
}
fn e457_c(&mut self,state:State)->Step {
let mut out=self.b457_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b457_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r70_c(out.state);
out
}
fn e458_c(&mut self,state:State)->Step {
let mut out=self.b458_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b458_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r71_c(out.state);
out
}
fn e459_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:459,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b459_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b459_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e460_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e461_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e462_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e463_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e464_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e460_c(&mut self,state:State)->Step {
let mut out=self.b460_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,460,60);
} else {self.display_failures(state.consumed.max(state.matched),&["'CAPTURE('"]);}
out
}
fn b460_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CAPTURE(",true,42,false,"CAPTURE(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CAPTURE('");}
out
}
fn e461_c(&mut self,state:State)->Step {
let mut out=self.b461_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,461,60);
out.events=self.event(Event::Capture {site:71,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b461_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e462_c(&mut self,state:State)->Step {
let mut out=self.b462_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,462,60);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b462_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e463_c(&mut self,state:State)->Step {
let mut out=self.b463_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:72,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b463_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e464_c(&mut self,state:State)->Step {
let mut out=self.b464_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,464,60);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b464_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e465_c(&mut self,state:State)->Step {
let mut out=self.b465_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b465_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e466_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e467_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e468_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e466_c(&mut self,state:State)->Step {
let mut out=self.b466_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,466,61);
} else {self.display_failures(state.consumed.max(state.matched),&["'SAME_AS('"]);}
out
}
fn b466_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"SAME_AS(",true,56,false,"SAME_AS(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'SAME_AS('");}
out
}
fn e467_c(&mut self,state:State)->Step {
let mut out=self.b467_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,467,61);
out.events=self.event(Event::Capture {site:73,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b467_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e468_c(&mut self,state:State)->Step {
let mut out=self.b468_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,468,61);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b468_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e469_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:469,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b469_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b469_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e470_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e471_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e472_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e470_c(&mut self,state:State)->Step {
let mut out=self.b470_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,470,62);
} else {self.display_failures(state.consumed.max(state.matched),&["'CAPTURE_SCOPE('"]);}
out
}
fn b470_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CAPTURE_SCOPE(",true,43,false,"CAPTURE_SCOPE(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CAPTURE_SCOPE('");}
out
}
fn e471_c(&mut self,state:State)->Step {
let mut out=self.b471_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:74,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b471_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e472_c(&mut self,state:State)->Step {
let mut out=self.b472_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,472,62);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b472_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e473_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:473,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b473_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b473_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e474_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e475_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e476_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e480_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e474_c(&mut self,state:State)->Step {
let mut out=self.b474_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,474,63);
} else {self.display_failures(state.consumed.max(state.matched),&["'COMMIT('"]);}
out
}
fn b474_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"COMMIT(",true,47,false,"COMMIT(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'COMMIT('");}
out
}
fn e475_c(&mut self,state:State)->Step {
let mut out=self.b475_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:75,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b475_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e476_c(&mut self,state:State)->Step {
let mut out=self.b476_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b476_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e477_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e477_c(&mut self,state:State)->Step {
let mut out=self.b477_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b477_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e478_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e479_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e478_c(&mut self,state:State)->Step {
let mut out=self.b478_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,478,63);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b478_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e479_c(&mut self,state:State)->Step {
let mut out=self.b479_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,479,63);
out.events=self.event(Event::Capture {site:76,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b479_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e480_c(&mut self,state:State)->Step {
let mut out=self.b480_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,480,63);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b480_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e481_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:481,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b481_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b481_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e482_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e483_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e484_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e485_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e486_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e482_c(&mut self,state:State)->Step {
let mut out=self.b482_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,482,64);
} else {self.display_failures(state.consumed.max(state.matched),&["'LABEL('"]);}
out
}
fn b482_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"LABEL(",true,51,false,"LABEL(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'LABEL('");}
out
}
fn e483_c(&mut self,state:State)->Step {
let mut out=self.b483_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,483,64);
out.events=self.event(Event::Capture {site:77,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b483_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e484_c(&mut self,state:State)->Step {
let mut out=self.b484_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,484,64);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b484_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e485_c(&mut self,state:State)->Step {
let mut out=self.b485_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:78,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b485_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e486_c(&mut self,state:State)->Step {
let mut out=self.b486_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,486,64);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b486_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e487_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:487,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b487_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b487_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e488_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e489_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e490_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e491_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e492_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e496_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e488_c(&mut self,state:State)->Step {
let mut out=self.b488_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,488,65);
} else {self.display_failures(state.consumed.max(state.matched),&["'CATCH('"]);}
out
}
fn b488_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CATCH(",true,44,false,"CATCH(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CATCH('");}
out
}
fn e489_c(&mut self,state:State)->Step {
let mut out=self.b489_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:79,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b489_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e490_c(&mut self,state:State)->Step {
let mut out=self.b490_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,490,65);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b490_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e491_c(&mut self,state:State)->Step {
let mut out=self.b491_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:80,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b491_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e492_c(&mut self,state:State)->Step {
let mut out=self.b492_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b492_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e493_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e493_c(&mut self,state:State)->Step {
let mut out=self.b493_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b493_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e494_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e495_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e494_c(&mut self,state:State)->Step {
let mut out=self.b494_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,494,65);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b494_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e495_c(&mut self,state:State)->Step {
let mut out=self.b495_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,495,65);
out.events=self.event(Event::Capture {site:81,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b495_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e496_c(&mut self,state:State)->Step {
let mut out=self.b496_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,496,65);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b496_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e497_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:497,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b497_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b497_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e498_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e499_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e500_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e498_c(&mut self,state:State)->Step {
let mut out=self.b498_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,498,66);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b498_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e499_c(&mut self,state:State)->Step {
let mut out=self.b499_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:82,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b499_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e500_c(&mut self,state:State)->Step {
let mut out=self.b500_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,500,66);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b500_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e501_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:501,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b501_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b501_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e502_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e503_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e504_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e502_c(&mut self,state:State)->Step {
let mut out=self.b502_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,502,67);
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b502_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,59,false,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e503_c(&mut self,state:State)->Step {
let mut out=self.b503_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:83,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b503_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e504_c(&mut self,state:State)->Step {
let mut out=self.b504_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,504,67);
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b504_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,60,false,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e505_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:505,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b505_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b505_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e506_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e507_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e508_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e506_c(&mut self,state:State)->Step {
let mut out=self.b506_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,506,68);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b506_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,78,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e507_c(&mut self,state:State)->Step {
let mut out=self.b507_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:84,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b507_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r74_c(out.state);
out
}
fn e508_c(&mut self,state:State)->Step {
let mut out=self.b508_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,508,68);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b508_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,80,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e509_c(&mut self,state:State)->Step {
let mut out=self.b509_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b509_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e510_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e511_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e512_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e513_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e510_c(&mut self,state:State)->Step {
let mut out=self.b510_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,510,69);
} else {self.display_failures(state.consumed.max(state.matched),&["'ERROR'"]);}
out
}
fn b510_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ERROR",true,50,true,"ERROR");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'ERROR'");}
out
}
fn e511_c(&mut self,state:State)->Step {
let mut out=self.b511_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,511,69);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b511_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e512_c(&mut self,state:State)->Step {
let mut out=self.b512_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,512,69);
out.events=self.event(Event::Capture {site:85,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b512_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e513_c(&mut self,state:State)->Step {
let mut out=self.b513_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,513,69);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b513_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e514_c(&mut self,state:State)->Step {
let mut out=self.b514_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b514_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e515_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e515_c(&mut self,state:State)->Step {
let mut out=self.b515_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,515,70);
out.events=self.event(Event::Capture {site:86,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b515_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"STRING","STRINGParser");
out
}
fn e516_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:516,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b516_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b516_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e517_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e518_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e522_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e517_c(&mut self,state:State)->Step {
let mut out=self.b517_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,517,71);
} else {self.display_failures(state.consumed.max(state.matched),&["NOT_OWNED_CALLParser"]);}
out
}
fn b517_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"NOT_OWNED_CALL","NOT_OWNED_CALLParser");
out
}
fn e518_c(&mut self,state:State)->Step {
let mut out=self.b518_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'", "__CaptureSite"]);}
out
}
fn b518_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e519_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:87,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e519_c(&mut self,state:State)->Step {
let mut out=self.b519_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b519_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e520_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e521_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e520_c(&mut self,state:State)->Step {
let mut out=self.b520_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,520,71);
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser"]);}
out
}
fn b520_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e521_c(&mut self,state:State)->Step {
let mut out=self.b521_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,521,71);
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b521_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,".",true,6,false,".");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'.'");}
out
}
fn e522_c(&mut self,state:State)->Step {
let mut out=self.b522_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,522,71);
out.events=self.event(Event::Capture {site:88,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b522_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e523_c(&mut self,state:State)->Step {
let mut out=self.b523_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b523_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e524_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e525_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e526_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e527_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e524_c(&mut self,state:State)->Step {
let mut out=self.b524_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,524,72);
} else {self.display_failures(state.consumed.max(state.matched),&["'@typeof'"]);}
out
}
fn b524_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@typeof",true,36,true,"@typeof");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@typeof'");}
out
}
fn e525_c(&mut self,state:State)->Step {
let mut out=self.b525_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,525,72);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b525_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e526_c(&mut self,state:State)->Step {
let mut out=self.b526_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,526,72);
out.events=self.event(Event::Capture {site:89,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b526_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e527_c(&mut self,state:State)->Step {
let mut out=self.b527_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,527,72);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b527_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e528_c(&mut self,state:State)->Step {
let mut out=self.b528_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b528_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e529_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e530_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e529_c(&mut self,state:State)->Step {
let mut out=self.b529_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,529,73);
out.events=self.event(Event::Capture {site:90,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b529_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e530_c(&mut self,state:State)->Step {
let mut out=self.b530_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b530_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e531_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e531_c(&mut self,state:State)->Step {
let mut out=self.b531_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b531_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e532_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e533_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e532_c(&mut self,state:State)->Step {
let mut out=self.b532_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,532,73);
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b532_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,".",true,6,false,".");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'.'");}
out
}
fn e533_c(&mut self,state:State)->Step {
let mut out=self.b533_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,533,73);
out.events=self.event(Event::Capture {site:91,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b533_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e534_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:534,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b534_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b534_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e535_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e535_c(&mut self,state:State)->Step {
let mut out=self.b535_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b535_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r52_c(out.state);
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
fn guard_e48_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e48_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"=");
self.display_failures(t1.consumed.max(t1.matched),&["'='", "'='"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'='"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e51_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e51_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"::=");
self.display_failures(t1.consumed.max(t1.matched),&["'::='", "'::='"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'::='"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e76_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e76_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"{");
self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'{'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'}'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e89_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e89_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"(");
self.display_failures(t1.consumed.max(t1.matched),&["'('", "'('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e93_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e93_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"[");
self.display_failures(t1.consumed.max(t1.matched),&["'['", "'['"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'['", "']'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e97_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e97_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"{");
self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'{'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'}'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e101_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e101_c(state);}
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
fn guard_e109_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e109_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CAPTURE");
self.display_failures(t1.consumed.max(t1.matched),&["'CAPTURE'", "'CAPTURE'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "','", "'CAPTURE'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e116_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e116_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"SAME_AS");
self.display_failures(t1.consumed.max(t1.matched),&["'SAME_AS'", "'SAME_AS'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "'SAME_AS'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e121_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e121_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CHAR_RANGE");
self.display_failures(t1.consumed.max(t1.matched),&["'CHAR_RANGE'", "'CHAR_RANGE'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "','", "'CHAR_RANGE'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e128_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e128_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"NEGATION");
self.display_failures(t1.consumed.max(t1.matched),&["'NEGATION'", "'NEGATION'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "'NEGATION'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e136_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e136_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"UNTIL");
let d3=self.diag_rule(16,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'UNTIL'", "'UNTIL'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'UNTIL'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e137_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e137_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"NEGATION");
let d3=self.diag_rule(17,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'NEGATION'", "'NEGATION'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'NEGATION'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e138_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e138_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"LOOKAHEAD");
let d3=self.diag_rule(18,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'LOOKAHEAD'", "'LOOKAHEAD'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'LOOKAHEAD'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e139_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e139_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"NEGATIVE_LOOKAHEAD");
let d3=self.diag_rule(19,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'NEGATIVE_LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'NEGATIVE_LOOKAHEAD'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e140_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e140_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CHAR_RANGE");
let d3=self.diag_rule(20,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CHAR_RANGE'", "'CHAR_RANGE'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CHAR_RANGE'", "'('", "','", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e141_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e141_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CI");
let d3=self.diag_rule(21,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CI'", "'CI'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CI'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e142_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e142_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"ADAPTER");
let d3=self.diag_rule(22,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'ADAPTER'", "'ADAPTER'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'ADAPTER'", "'('", "','", "'version'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e143_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e143_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"ANY");
let d3=self.diag_rule(23,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'ANY'", "'ANY'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'ANY'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e144_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e144_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"EOF");
let d3=self.diag_rule(24,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'EOF'", "'EOF'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'EOF'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e145_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e145_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"EMPTY");
let d3=self.diag_rule(25,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'EMPTY'", "'EMPTY'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'EMPTY'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e203_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e203_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@root");
let d3=self.diag_rule(28,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@root'", "'@root'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@root'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e204_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e204_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@mapping");
let d3=self.diag_rule(29,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@mapping'", "'@mapping'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@mapping'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e205_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e205_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@eval");
let d3=self.diag_rule(30,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@eval'", "'@eval'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@eval'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e206_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e206_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@whitespace");
let d3=self.diag_rule(31,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@whitespace'", "'@whitespace'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@whitespace'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e207_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e207_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@interleave");
let d3=self.diag_rule(33,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@interleave'", "'@interleave'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@interleave'", "'('", "'profile'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e208_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e208_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@lexicalContext");
let d3=self.diag_rule(32,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@lexicalContext'", "'@lexicalContext'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@lexicalContext'", "'('", "'tokens'", "'='", "'['", "']'", "')'"]);}
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
let d2=self.diag_fail(t1.position::<false>(),"@uniqueLongestChoice");
let d3=self.diag_rule(41,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@uniqueLongestChoice'", "'@uniqueLongestChoice'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@uniqueLongestChoice'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e217_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e217_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@namePredicate");
let d3=self.diag_rule(42,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@namePredicate'", "'@namePredicate'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@namePredicate'", "'('", "'snapshot'", "'='", "','", "'version'", "'name'", "'kind'", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e218_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e218_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@predicate");
let d3=self.diag_rule(43,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@predicate'", "'@predicate'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@predicate'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e219_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e219_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@predictiveChoice");
let d3=self.diag_rule(44,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@predictiveChoice'", "'@predictiveChoice'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@predictiveChoice'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e220_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e220_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@precedence");
let d3=self.diag_rule(45,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@precedence'", "'@precedence'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@precedence'", "'('", "'level'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e221_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e221_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@doc");
let d3=self.diag_rule(46,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@doc'", "'@doc'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@doc'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e222_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e222_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@recovery");
let d3=self.diag_rule(47,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@recovery'", "'@recovery'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@recovery'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e223_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e223_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@skip");
let d3=self.diag_rule(48,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@skip'", "'@skip'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@skip'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e224_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e224_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@commonField");
let d3=self.diag_rule(49,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@commonField'", "'@commonField'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@commonField'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e225_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e225_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@enum");
let d3=self.diag_rule(50,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@enum'", "'@enum'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@enum'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e226_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e226_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@");
let d3=self.diag_rule(51,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@'", "'@'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e383_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e383_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"sync");
self.display_failures(t1.consumed.max(t1.matched),&["'sync'", "'sync'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'='", "'sync'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e447_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e447_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CAPTURE(");
let d3=self.diag_rule(60,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CAPTURE('", "'CAPTURE('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CAPTURE('", "','", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e448_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e448_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"SAME_AS(");
let d3=self.diag_rule(61,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'SAME_AS('", "'SAME_AS('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'SAME_AS('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e449_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e449_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CAPTURE_SCOPE(");
let d3=self.diag_rule(62,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CAPTURE_SCOPE('", "'CAPTURE_SCOPE('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CAPTURE_SCOPE('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e450_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e450_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"COMMIT(");
let d3=self.diag_rule(63,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'COMMIT('", "'COMMIT('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'COMMIT('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e451_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e451_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"LABEL(");
let d3=self.diag_rule(64,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'LABEL('", "'LABEL('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'LABEL('", "','", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e452_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e452_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CATCH(");
let d3=self.diag_rule(65,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CATCH('", "'CATCH('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CATCH('", "','", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e453_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e453_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"(");
let d3=self.diag_rule(66,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'('", "'('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e454_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e454_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"[");
let d3=self.diag_rule(67,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'['", "'['"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'['", "']'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e455_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e455_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"{");
let d3=self.diag_rule(68,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'{'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'}'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e456_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e456_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"ERROR");
let d3=self.diag_rule(69,d2);
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
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::NOT,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"CAPTURE(",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::NOT,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"SAME_AS(",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::NOT,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"CAPTURE_SCOPE(",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::NOT,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"COMMIT(",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::NOT,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"LABEL(",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::NOT,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"CATCH(",min:0,max:0,children:&[]}]}]}]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_4<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"'",min:0,max:0,children:&[]},lexical_program::LexicalExpression {op:lexical_program::Op::REPEAT,text:"",min:0,max:-1,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"\\",min:0,max:0,children:&[]},lexical_program::LexicalExpression {op:lexical_program::Op::ANY,text:"",min:0,max:0,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::EXCEPT,text:"'\\",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"'",min:0,max:0,children:&[]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_5<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:97,max:122,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:65,max:90,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"_",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::REPEAT,text:"",min:0,max:-1,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:97,max:122,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:65,max:90,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"_",min:0,max:0,children:&[]}]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:48,max:57,children:&[]}]}]}]}]}]}]}]}]}]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_6<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
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
