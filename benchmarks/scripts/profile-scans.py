#!/usr/bin/env python3
"""専用の生成 JSON crate にだけ走査計数を追加する。通常の時間 A/B とは別に使う。"""
import json
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
ir = json.loads(pathlib.Path(sys.argv[2]).read_text())
exprs = ir['semantics']['expressions']
classes = {x['exprId'] for x in ir['plan']['scans'] if x.get('class')}
parser = root / 'parser.rs'
source = parser.read_text()
labels = []

def wrap(m):
    i = int(m[1])
    labels.append((i, exprs[i]['id'], exprs[i]['id'] in classes))
    return f'''fn s{i}(&self,p:usize)->Option<usize> {{
        super::scan_profile::enter();
        let start=std::time::Instant::now();
        let result=self.profile_s{i}(p);
        super::scan_profile::leave({i},result.unwrap_or(0),self.text.as_bytes().get(p)==Some(&b'\\\\'),start);
        result
    }}
    #[allow(unused_mut)]
    fn profile_s{i}(&self,mut p:usize)->Option<usize> {{'''

# bulk の 1 区間も別に計数する。関数全体の長さには escape を含む複数区間が
# 混じるため、それを連続した走査長と読み替えない。
segments = list(re.finditer(r'let bytes=self\.text\.as_bytes\(\);(while[^\n]*?\{)', source))
for m in reversed(segments):
    i = int(list(re.finditer(r'fn s(\d+)\(', source[:m.start()]))[-1][1])
    begin = m.end()-1
    depth = 1
    end = begin+1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    loop = source[m.start()+len('let bytes=self.text.as_bytes();'):end]
    replacement = f'''let bytes=self.text.as_bytes();
        let profile_begin=p;
        super::scan_profile::enter();
        let profile_clock=std::time::Instant::now();
        (|| {{ {loop} }})();
        super::scan_profile::leave({10000+i},p-profile_begin,bytes.get(p)==Some(&b'\\\\'),profile_clock);'''
    source = source[:m.start()] + replacement + source[end:]
    labels.append((10000+i, exprs[i]['id']+':bulk-segment', True))

source, count = re.subn(r'fn s(\d+)\(&self,mut p:usize\)->Option<usize> \{', wrap, source)
assert count, '走査関数が無い'
parser.write_text(source)
mod = root / 'mod.rs'
mod.write_text(mod.read_text() + '\npub mod scan_profile;\n')
(root / 'scan_profile.rs').write_text('''use std::{cell::RefCell, collections::BTreeMap, time::Instant};
type Key=(usize,usize,bool);
#[derive(Default)]
struct Data { stack:Vec<u128>, values:BTreeMap<Key,(u64,u128,u128)> }
thread_local! { static DATA:RefCell<Data>=RefCell::new(Data::default()); }
pub fn enter() { DATA.with(|d|d.borrow_mut().stack.push(0)); }
pub fn leave(id:usize,len:usize,escape:bool,start:Instant) {
    let elapsed=start.elapsed().as_nanos();
    DATA.with(|d| {
        let mut d=d.borrow_mut();
        let children=d.stack.pop().unwrap();
        if let Some(parent)=d.stack.last_mut() { *parent+=elapsed; }
        let bucket=match len {0=>0,1..=7=>1,_=>2};
        let value=d.values.entry((id,bucket,escape)).or_default();
        value.0+=1;value.1+=elapsed.saturating_sub(children);value.2+=len as u128;
    });
}
pub fn clear() { DATA.with(|d| *d.borrow_mut()=Data::default()); }
pub fn report() {
    println!("id,bucket,escape,calls,self_ns,bytes");
    DATA.with(|d| for ((id,bucket,escape),(calls,ns,bytes)) in &d.borrow().values {
        println!("{id},{bucket},{escape},{calls},{ns},{bytes}");
    });
}
''')
(root / 'scan-labels.json').write_text(json.dumps(labels, ensure_ascii=False, indent=2)+'\n')
