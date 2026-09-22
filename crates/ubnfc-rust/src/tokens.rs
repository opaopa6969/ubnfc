use crate::{emit::Generator, EmitError};
use std::fmt::Write;
use ubnfc_front::ir::*;
pub(crate) fn generate(g: &Generator<'_>) -> Result<String, EmitError> {
    let mut s = String::new();
    for (i, t) in g.ir.semantics.tokens.iter().enumerate() {
        let label =
            g.ir.semantics
                .symbols
                .iter()
                .find(|symbol| symbol.id == t.symbol_id)
                .map_or(t.symbol_id.as_str(), |s| s.name.as_str());
        writeln!(
            s,
            "fn token_{i}<const MATCH:bool>(&mut self,state:State)->Step {{"
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
            writeln!(s,"_=>None,}};let Some(next)=next else {{break;}};current=next;p+=n;}}self.primitive::<MATCH>(state,end.map(|p|p-start),{label:?})\n}}").unwrap();
            continue;
        }
        match &t.spec {
            TokenSpec::Builtin{which,code_points}=>{
                match which {
                    BuiltinToken::Number=>writeln!(s,"self.number::<MATCH>(state,{label:?})").unwrap(),
                    BuiltinToken::Identifier=>writeln!(s,"self.identifier::<MATCH>(state,{label:?})").unwrap(),
                    BuiltinToken::SingleQuoted|BuiltinToken::Quoted=>writeln!(s,"self.quoted::<MATCH>(state,{:?},{label:?})",if *which==BuiltinToken::Quoted {'"'} else {'\''}).unwrap(),
                    BuiltinToken::EndOfSource=>writeln!(s,"self.eof::<MATCH>(state,{label:?})").unwrap(),
                    BuiltinToken::Space=>{writeln!(s,"let length=self.text.as_bytes().get(state.position::<MATCH>()).filter(|b|matches!(b,9..=13|32)).map(|_|1);self.primitive::<MATCH>(state,length,{label:?})").unwrap();},
                    _=>{
                        let condition=match which {
                            BuiltinToken::Digit=>"c.is_ascii_digit()".into(),BuiltinToken::Alphabet=>"c.is_ascii_alphabetic()".into(),BuiltinToken::AlphabetUnderScore=>"c.is_ascii_alphabetic() || *c=='_'".into(),BuiltinToken::AlphabetNumericUnderScore=>"c.is_ascii_alphanumeric() || *c=='_'".into(),BuiltinToken::CodePointSet=>format!("{:?}.contains(&(*c as u64))",code_points),_=>unreachable!()
                        };
                        writeln!(s,"let length=self.input.cp_at(state.position::<MATCH>()).filter(|(c,_)|{condition}).map(|(_,n)|n);self.primitive::<MATCH>(state,length,{label:?})").unwrap();
                    }
                }
            },
            TokenSpec::Any{}=>writeln!(s,"self.any::<MATCH>(state,{label:?})").unwrap(),
            TokenSpec::Eof{}=>writeln!(s,"self.eof::<MATCH>(state,{label:?})").unwrap(),
            TokenSpec::Empty{}=>s.push_str("self.empty(state)\n"),
            TokenSpec::Until{terminator,..}=>writeln!(s,"self.until::<MATCH>(state,{terminator:?})").unwrap(),
            TokenSpec::CaseInsensitive{word,..}=>writeln!(s,"self.literal::<MATCH>(state,{word:?},false,{label:?})").unwrap(),
            TokenSpec::CharRange{min,max}=>writeln!(s,"let length=self.input.cp_at(state.position::<MATCH>()).filter(|(c,_)|({min}..={max}).contains(&(*c as u64))).map(|(_,n)|n);self.primitive::<MATCH>(state,length,{label:?})").unwrap(),
            TokenSpec::Negation{excluded}=>writeln!(s,"let length=self.input.cp_at(state.position::<MATCH>()).filter(|(c,_)|!{excluded:?}.contains(*c)).map(|(_,n)|n);self.primitive::<MATCH>(state,length,{label:?})").unwrap(),
            TokenSpec::Lookahead{pattern}|TokenSpec::NegativeLookahead{pattern}=>{
                writeln!(s,"let child=self.literal::<true>(state.begin(),{pattern:?},true,{label:?});").unwrap();
                if matches!(t.spec,TokenSpec::NegativeLookahead{..}) {writeln!(s,"if child.ok {{self.fail(state,state.matched,{label:?})}} else {{Step::yes(state.commit(state.begin()))}}").unwrap();}
                else {s.push_str("Step {state:if child.ok {state.commit(child.state)} else {state},events:EventId(0),..child}\n");}
            },
            TokenSpec::Extern{logical_id,..}=>writeln!(s,"self.scan::<MATCH>(state,{logical_id:?},{label:?})").unwrap(),
            TokenSpec::Regex{..}=>unreachable!("validated DFA"),
        }
        s.push_str("}\n");
    }
    for (i, p) in g.ir.semantics.trivia.policies.iter().enumerate() {
        writeln!(
            s,
            "fn trivia_{i}<const MATCH:bool>(&mut self,mut state:State)->State {{"
        )
        .unwrap();
        if g.ir.plan.trivia_cache.enabled {
            writeln!(s,"let key=Key{{expression:{i},state,matched_mode:MATCH,version:0}};if let Some(hit)=self.trivia_cache.get(key) {{return hit;}}").unwrap();
        }
        s.push_str("let start=state.position::<MATCH>();let mut p=start;loop {let before=p;\n");
        for d in &p.effective_delimiter_set {
            match d {
                Delimiter::Characters{code_points}=>writeln!(s,"while let Some((c,n))=self.input.cp_at(p) {{if !{:?}.contains(&(c as u64)) {{break;}}p+=n;}}",code_points).unwrap(),
                Delimiter::LineComment{open,..}=>writeln!(s,"if self.input.starts_with(p,{open:?}) {{p+={};while let Some((c,n))=self.input.cp_at(p) {{if matches!(c,'\\n'|'\\r') {{break;}}p+=n;}}}}",open.len()).unwrap(),
                Delimiter::BlockComment{open,close}=>writeln!(s,"if self.input.starts_with(p,{open:?}) {{let body=p+{};if let Some(n)=self.text[body..].find({close:?}) {{p=body+n+{};}}}}",open.len(),close.len()).unwrap(),
            }
        }
        s.push_str("if p==before {break;}}state.advance::<MATCH>(p-start);\n");
        if g.ir.plan.trivia_cache.enabled {
            s.push_str("self.trivia_cache.insert(key,state);\n");
        }
        s.push_str("state}\n");
    }
    Ok(s)
}
