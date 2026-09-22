use crate::{emit::Generator, EmitError};
use std::fmt::Write;
use ubnfc_front::ir::*;
pub(crate) fn generate(g: &Generator<'_>) -> Result<String, EmitError> {
    let mut s = String::new();
    for (i, t) in g.ir.semantics.tokens.iter().enumerate() {
        writeln!(
            s,
            "fn token_{i}<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {{let _=label;let out={{"
        )
        .unwrap();
        let dfa =
            g.ir.plan
                .lexical
                .iter()
                .find(|p| p.token_id == t.symbol_id && p.strategy == LexicalPlanStrategy::Dfa)
                .and_then(|p| p.dfa.as_ref());
        if let Some(dfa) = dfa {
            writeln!(s,"let start=state.position::<MATCH>();let mut p=start;let mut current={}u64;let mut end=None;loop {{",dfa.start_state).unwrap();
            let accepting = dfa
                .states
                .iter()
                .filter(|st| st.accepting)
                .map(|st| st.id)
                .collect::<Vec<_>>();
            writeln!(s,"if p>start && {accepting:?}.contains(&current) {{end=Some(p);}}let Some((c,n))=self.input.cp_at(p) else {{break;}};let next=match (current,c as u32) {{").unwrap();
            for st in &dfa.states {
                for tr in &st.transitions {
                    writeln!(
                        s,
                        "({},{}..={})=>Some({}),",
                        st.id, tr.range.start, tr.range.end, tr.target
                    )
                    .unwrap();
                }
            }
            writeln!(s,"_=>None,}};let Some(next)=next else {{break;}};current=next;p+=n;}}self.primitive::<MATCH>(state,end.map(|p|p-start),label)}};if !out.ok {{self.display.fail(state.consumed.max(state.matched),display);}}out\n}}").unwrap();
            continue;
        }
        match &t.spec {
            TokenSpec::Builtin{which,code_points}=>{
                match which {
                    BuiltinToken::Number=>writeln!(s,"self.number::<MATCH>(state,label)").unwrap(),
                    BuiltinToken::Identifier=>writeln!(s,"self.identifier::<MATCH>(state,label)").unwrap(),
                    BuiltinToken::SingleQuoted|BuiltinToken::Quoted=>writeln!(s,"self.quoted::<MATCH>(state,{:?},label)",if *which==BuiltinToken::Quoted {'"'} else {'\''}).unwrap(),
                    BuiltinToken::EndOfSource=>writeln!(s,"self.eof::<MATCH>(state,label)").unwrap(),
                    BuiltinToken::Space=>{writeln!(s,"let length=self.text.as_bytes().get(state.position::<MATCH>()).filter(|b|matches!(b,9..=13|32)).map(|_|1);self.primitive::<MATCH>(state,length,label)").unwrap();},
                    _=>{
                        let condition=match which {
                            BuiltinToken::Digit=>"c.is_ascii_digit()".into(),BuiltinToken::Alphabet=>"c.is_ascii_alphabetic()".into(),BuiltinToken::AlphabetUnderScore=>"c.is_ascii_alphabetic() || *c=='_'".into(),BuiltinToken::AlphabetNumericUnderScore=>"c.is_ascii_alphanumeric() || *c=='_'".into(),BuiltinToken::CodePointSet=>format!("{:?}.contains(&(*c as u64))",code_points),_=>unreachable!()
                        };
                        writeln!(s,"let length=self.input.cp_at(state.position::<MATCH>()).filter(|(c,_)|{condition}).map(|(_,n)|n);self.primitive::<MATCH>(state,length,label)").unwrap();
                    }
                }
            },
            TokenSpec::Any{}=>writeln!(s,"self.any::<MATCH>(state,label)").unwrap(),
            TokenSpec::Eof{}=>writeln!(s,"self.eof::<MATCH>(state,label)").unwrap(),
            TokenSpec::Empty{}=>s.push_str("self.empty(state)\n"),
            TokenSpec::Until{terminator,..}=>writeln!(s,"self.until::<MATCH>(state,{terminator:?},{:?})",format!("'{terminator}'")).unwrap(),
            TokenSpec::CaseInsensitive{word,..}=>writeln!(s,"self.literal::<MATCH>(state,{word:?},false,label)").unwrap(),
            TokenSpec::CharRange{min,max}=>writeln!(s,"let length=self.input.cp_at(state.position::<MATCH>()).filter(|(c,_)|({min}..={max}).contains(&(*c as u64))).map(|(_,n)|n);self.primitive::<MATCH>(state,length,label)").unwrap(),
            TokenSpec::Negation{excluded}=>writeln!(s,"let length=self.input.cp_at(state.position::<MATCH>()).filter(|(c,_)|!{excluded:?}.contains(*c)).map(|(_,n)|n);self.primitive::<MATCH>(state,length,label)").unwrap(),
            TokenSpec::Lookahead{pattern}|TokenSpec::NegativeLookahead{pattern}=>{
                if pattern.is_empty() { s.push_str("let child=self.fail(state.begin(),state.begin().matched,label);\n"); } else { writeln!(s,"let child=self.literal::<true>(state.begin(),{pattern:?},true,label);").unwrap(); }
                if matches!(t.spec,TokenSpec::NegativeLookahead{..}) {writeln!(s,"if child.ok {{self.fail(state,state.matched,label)}} else {{Step::yes(state.commit(state.begin()))}}").unwrap();}
                else {s.push_str("Step {state:if child.ok {state.commit(child.state)} else {state},events:EventId(0),..child}\n");}
            },
            TokenSpec::Extern{logical_id,..}=>writeln!(s,"self.scan::<MATCH>(state,{logical_id:?},label)").unwrap(),
            TokenSpec::Regex{..}=>unreachable!("validated DFA"),
        }
        if matches!(t.spec, TokenSpec::Extern { .. }) {
            s.push_str("};let _=display;out}\n");
        } else {
            s.push_str("};if !out.ok {self.display.fail(state.consumed.max(state.matched),display);}out}\n");
        }
    }
    for (i, p) in g.ir.semantics.trivia.policies.iter().enumerate() {
        writeln!(
            s,
            "fn trivia_{i}<const MATCH:bool>(&mut self,state:State)->Step {{"
        )
        .unwrap();
        if g.ir.plan.trivia_cache.enabled {
            writeln!(s,"let key=Key{{expression:{i},state,matched_mode:MATCH,version:0}};if let Some((hit,display))=self.trivia_cache.get(key) {{self.display.merge(display);return hit;}}").unwrap();
        }
        s.push_str(if g.ir.plan.trivia_cache.enabled {
            "self.display.enter(state.consumed.max(state.matched));let mut out=Step::yes(state);loop {let before=out.state.position::<MATCH>();\n"
        } else {
            "self.display.reach(state.consumed.max(state.matched));let mut out=Step::yes(state);loop {let before=out.state.position::<MATCH>();\n"
        });
        // 先頭 byte がどの区切りの先頭でもなければ、下の各 block は何も進めず候補 label を
        // 記録して終わる。その label 列を 1 回の failures で記録して抜ける（記録内容・位置は同じ）。
        if let Some(bytes) = crate::predict::trivia_first_bytes(p) {
            let mut labels = Vec::new();
            let mut diag = String::new();
            for d in &p.effective_delimiter_set {
                match d {
                    Delimiter::Characters { .. } => labels.push("' '".to_owned()),
                    Delimiter::LineComment { open, .. } => {
                        labels.push(format!("'{open}'"));
                        if p.effective_delimiter_set.len() == 1 {
                            let label = format!("'{open}'");
                            diag = format!("let d=self.diag.fail(out.state.consumed.max(out.state.matched),{label:?});out.diag=self.diag.join(out.diag,d);");
                        }
                    }
                    Delimiter::BlockComment { open, close } => {
                        labels.push(format!("'{open}'"));
                        labels.push(format!("'{close}'"));
                    }
                }
            }
            let pattern = bytes
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join("|");
            if pattern.is_empty() {
                writeln!(s,"if true {{{diag}self.display.failures(out.state.consumed.max(out.state.matched),&{labels:?});break;}}").unwrap();
            } else {
                writeln!(s,"if !matches!(self.text.as_bytes().get(before),Some({pattern})) {{{diag}self.display.failures(out.state.consumed.max(out.state.matched),&{labels:?});break;}}").unwrap();
            }
        }
        for d in &p.effective_delimiter_set {
            s.push_str("{let start=out.state.position::<MATCH>();let mut p=start;\n");
            match d {
                Delimiter::Characters{code_points}=>writeln!(s,"while let Some((c,n))=self.input.cp_at(p) {{if !{:?}.contains(&(c as u64)) {{break;}}p+=n;}}if p>start {{out.state.advance::<MATCH>(p-start);continue;}}",code_points).unwrap(),
                Delimiter::LineComment{open,..}=>{
                    writeln!(s,"if self.input.starts_with(p,{open:?}) {{p+={};while let Some((c,n))=self.input.cp_at(p) {{if matches!(c,'\\n'|'\\r') {{break;}}p+=n;}}let end=if self.input.starts_with(p,\"\\r\\n\") {{p+2}} else {{self.input.cp_at(p).map_or(p,|(_,n)|p+n)}};out.state.advance::<MATCH>(p-start);if !MATCH {{out.state.matched=end;}}continue;}}",open.len()).unwrap();
                    if p.effective_delimiter_set.len()==1 {
                        let label=format!("'{open}'");
                        writeln!(s,"let d=self.diag.fail(out.state.consumed.max(out.state.matched),{label:?});out.diag=self.diag.join(out.diag,d);").unwrap();
                    }
                },
                Delimiter::BlockComment{open,close}=>writeln!(s,"if self.input.starts_with(p,{open:?}) {{let body=p+{};if let Some(n)=self.text[body..].find({close:?}) {{p=body+n+{};out.state.advance::<MATCH>(p-start);continue;}}}}",open.len(),close.len()).unwrap(),
            }
            match d {
                Delimiter::Characters { .. } => s.push_str("self.display.fail(out.state.consumed.max(out.state.matched),\"' '\");\n"),
                Delimiter::LineComment { open, .. } => writeln!(s,"self.display.fail(out.state.consumed.max(out.state.matched),{:?});",format!("'{open}'")).unwrap(),
                Delimiter::BlockComment { open, close } => writeln!(s,"if self.input.starts_with(start,{open:?}) {{self.display.fail(self.text.len(),{:?});self.display.fail(self.text.len(),\"WildCardStringParser\");}} else {{self.display.fail(out.state.consumed.max(out.state.matched),{:?});self.display.fail(out.state.consumed.max(out.state.matched),{:?});}}",format!("'{close}'"),format!("'{open}'"),format!("'{close}'")).unwrap(),
            }
            s.push_str("}\n");
        }
        s.push_str("if out.state.position::<MATCH>()==before {break;}}\n");
        if g.ir.plan.trivia_cache.enabled {
            s.push_str("if self.trivia_cache.len()<self.options.limits.memo_entries {self.trivia_cache.insert(key,(out,self.display.snapshot()));}else {self.resource_failure.get_or_insert((state.consumed,\"trivia memo budget exceeded\"));}\n");
        }
        s.push_str(if g.ir.plan.trivia_cache.enabled {
            "self.display.reach(out.state.consumed.max(out.state.matched));self.display.leave();out}\n"
        } else {
            "self.display.reach(out.state.consumed.max(out.state.matched));out}\n"
        });
        // 診断を記録しない読み飛ばし。候補除外の判定位置（predict.rs）にだけ使う。
        if p.effective_delimiter_set.is_empty() {
            writeln!(
                s,
                "fn skip_{i}<const MATCH:bool>(&self,state:State)->State {{state}}"
            )
            .unwrap();
            continue;
        }
        writeln!(s,"fn skip_{i}<const MATCH:bool>(&self,state:State)->State {{let mut out=state;loop {{let before=out.position::<MATCH>();").unwrap();
        if let Some(bytes) = crate::predict::trivia_first_bytes(p) {
            let pattern = bytes
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join("|");
            if !pattern.is_empty() {
                writeln!(
                    s,
                    "if !matches!(self.text.as_bytes().get(before),Some({pattern})) {{break;}}"
                )
                .unwrap();
            }
        }
        for d in &p.effective_delimiter_set {
            s.push_str("{let start=out.position::<MATCH>();let mut p=start;\n");
            match d {
                Delimiter::Characters{code_points}=>writeln!(s,"while let Some((c,n))=self.input.cp_at(p) {{if !{:?}.contains(&(c as u64)) {{break;}}p+=n;}}if p>start {{out.advance::<MATCH>(p-start);continue;}}",code_points).unwrap(),
                Delimiter::LineComment{open,..}=>writeln!(s,"if self.input.starts_with(p,{open:?}) {{p+={};while let Some((c,n))=self.input.cp_at(p) {{if matches!(c,'\\n'|'\\r') {{break;}}p+=n;}}let end=if self.input.starts_with(p,\"\\r\\n\") {{p+2}} else {{self.input.cp_at(p).map_or(p,|(_,n)|p+n)}};out.advance::<MATCH>(p-start);if !MATCH {{out.matched=end;}}continue;}}",open.len()).unwrap(),
                Delimiter::BlockComment{open,close}=>writeln!(s,"if self.input.starts_with(p,{open:?}) {{let body=p+{};if let Some(n)=self.text[body..].find({close:?}) {{p=body+n+{};out.advance::<MATCH>(p-start);continue;}}}}",open.len(),close.len()).unwrap(),
            }
            s.push_str("}\n");
        }
        s.push_str("if out.position::<MATCH>()==before {break;}}\nout}\n");
    }
    Ok(s)
}
